use crate::CitationStreamParser;
use crate::InlineHiddenTagParser;
use crate::InlineTagSpec;
use crate::ProposedPlanParser;
use crate::ProposedPlanSegment;
use crate::StreamTextChunk;
use crate::StreamTextParser;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct AssistantTextChunk {
    pub visible_text: String,
    pub citations: Vec<String>,
    pub plan_segments: Vec<ProposedPlanSegment>,
    pub completion_markers: usize,
}

impl AssistantTextChunk {
    pub fn is_empty(&self) -> bool {
        self.visible_text.is_empty()
            && self.citations.is_empty()
            && self.plan_segments.is_empty()
            && self.completion_markers == 0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CompletionMarkerTag {
    Complete,
}

pub const COMPLETION_MARKER: &str = "<pfterminal-task-complete></pfterminal-task-complete>";
const COMPLETION_MARKER_OPEN: &str = "<pfterminal-task-complete>";
const COMPLETION_MARKER_CLOSE: &str = "</pfterminal-task-complete>";

fn completion_marker_parser() -> InlineHiddenTagParser<CompletionMarkerTag> {
    InlineHiddenTagParser::new(vec![InlineTagSpec {
        tag: CompletionMarkerTag::Complete,
        open: COMPLETION_MARKER_OPEN,
        close: COMPLETION_MARKER_CLOSE,
    }])
}

/// Removes the host/model completion handshake from user-visible assistant text.
/// The literal tag is a mechanical protocol marker, not a semantic classifier.
pub fn strip_completion_markers(text: &str) -> (String, usize) {
    let mut parser = completion_marker_parser();
    let mut parsed = parser.push_str(text);
    let tail = parser.finish();
    parsed.visible_text.push_str(&tail.visible_text);
    parsed.extracted.extend(tail.extracted);
    (parsed.visible_text, parsed.extracted.len())
}

/// Parses assistant text streaming markup in one pass:
/// - strips `<oai-mem-citation>` tags and extracts citation payloads
/// - in plan mode, also strips `<proposed_plan>` blocks and emits plan segments
#[derive(Debug)]
pub struct AssistantTextStreamParser {
    plan_mode: bool,
    completion: InlineHiddenTagParser<CompletionMarkerTag>,
    citations: CitationStreamParser,
    plan: ProposedPlanParser,
}

impl Default for AssistantTextStreamParser {
    fn default() -> Self {
        Self {
            plan_mode: false,
            completion: completion_marker_parser(),
            citations: CitationStreamParser::default(),
            plan: ProposedPlanParser::default(),
        }
    }
}

impl AssistantTextStreamParser {
    pub fn new(plan_mode: bool) -> Self {
        Self {
            plan_mode,
            completion: completion_marker_parser(),
            ..Self::default()
        }
    }

    pub fn push_str(&mut self, chunk: &str) -> AssistantTextChunk {
        let completion_chunk = self.completion.push_str(chunk);
        let citation_chunk = self.citations.push_str(&completion_chunk.visible_text);
        let mut out = self.parse_visible_text(citation_chunk.visible_text);
        out.citations = citation_chunk.extracted;
        out.completion_markers = completion_chunk.extracted.len();
        out
    }

    pub fn finish(&mut self) -> AssistantTextChunk {
        let completion_chunk = self.completion.finish();
        let mut citation_chunk = self.citations.push_str(&completion_chunk.visible_text);
        let citation_tail = self.citations.finish();
        citation_chunk
            .visible_text
            .push_str(&citation_tail.visible_text);
        citation_chunk.extracted.extend(citation_tail.extracted);
        let mut out = self.parse_visible_text(citation_chunk.visible_text);
        if self.plan_mode {
            let mut tail = self.plan.finish();
            if !tail.is_empty() {
                out.visible_text.push_str(&tail.visible_text);
                out.plan_segments.append(&mut tail.extracted);
            }
        }
        out.citations = citation_chunk.extracted;
        out.completion_markers = completion_chunk.extracted.len();
        out
    }

    fn parse_visible_text(&mut self, visible_text: String) -> AssistantTextChunk {
        if !self.plan_mode {
            return AssistantTextChunk {
                visible_text,
                ..AssistantTextChunk::default()
            };
        }
        let plan_chunk: StreamTextChunk<ProposedPlanSegment> = self.plan.push_str(&visible_text);
        AssistantTextChunk {
            visible_text: plan_chunk.visible_text,
            plan_segments: plan_chunk.extracted,
            ..AssistantTextChunk::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::AssistantTextStreamParser;
    use super::COMPLETION_MARKER;
    use super::strip_completion_markers;
    use crate::ProposedPlanSegment;
    use pretty_assertions::assert_eq;

    #[test]
    fn parses_citations_across_seed_and_delta_boundaries() {
        let mut parser = AssistantTextStreamParser::new(/*plan_mode*/ false);

        let seeded = parser.push_str("hello <oai-mem-citation>doc");
        let parsed = parser.push_str("1</oai-mem-citation> world");
        let tail = parser.finish();

        assert_eq!(seeded.visible_text, "hello ");
        assert_eq!(seeded.citations, Vec::<String>::new());
        assert_eq!(parsed.visible_text, " world");
        assert_eq!(parsed.citations, vec!["doc1".to_string()]);
        assert_eq!(tail.visible_text, "");
        assert_eq!(tail.citations, Vec::<String>::new());
    }

    #[test]
    fn parses_plan_segments_after_citation_stripping() {
        let mut parser = AssistantTextStreamParser::new(/*plan_mode*/ true);

        let seeded = parser.push_str("Intro\n<proposed");
        let parsed = parser.push_str("_plan>\n- step <oai-mem-citation>doc</oai-mem-citation>\n");
        let tail = parser.push_str("</proposed_plan>\nOutro");
        let finish = parser.finish();

        assert_eq!(seeded.visible_text, "Intro\n");
        assert_eq!(
            seeded.plan_segments,
            vec![ProposedPlanSegment::Normal("Intro\n".to_string())]
        );
        assert_eq!(parsed.visible_text, "");
        assert_eq!(parsed.citations, vec!["doc".to_string()]);
        assert_eq!(
            parsed.plan_segments,
            vec![
                ProposedPlanSegment::ProposedPlanStart,
                ProposedPlanSegment::ProposedPlanDelta("- step \n".to_string()),
            ]
        );
        assert_eq!(tail.visible_text, "Outro");
        assert_eq!(
            tail.plan_segments,
            vec![
                ProposedPlanSegment::ProposedPlanEnd,
                ProposedPlanSegment::Normal("Outro".to_string()),
            ]
        );
        assert!(finish.is_empty());
    }

    #[test]
    fn strips_completion_marker_across_stream_boundaries() {
        let mut parser = AssistantTextStreamParser::new(/*plan_mode*/ false);
        let first = parser.push_str("Done. <pfterminal-task-");
        let second = parser.push_str("complete></pfterminal-task-complete>");
        let tail = parser.finish();

        assert_eq!(first.visible_text, "Done. ");
        assert_eq!(second.visible_text, "");
        assert_eq!(second.completion_markers, 1);
        assert!(tail.is_empty());
        assert_eq!(
            strip_completion_markers(&format!("ok{COMPLETION_MARKER}")),
            ("ok".to_string(), 1)
        );
    }
}

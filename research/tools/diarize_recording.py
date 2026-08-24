#!/usr/bin/env python3
import argparse
import json
from pathlib import Path

import mlx_whisper
import numpy as np
import soundfile as sf
import torch
from sklearn.cluster import AgglomerativeClustering
from sklearn.metrics import silhouette_score
from speechbrain.inference.speaker import EncoderClassifier


def hms(seconds):
    seconds = max(0, int(round(seconds)))
    h = seconds // 3600
    m = (seconds % 3600) // 60
    s = seconds % 60
    return f"{h:02d}:{m:02d}:{s:02d}"


def cluster_labels(embeddings, min_speakers=2, max_speakers=6):
    if len(embeddings) < 2:
        return np.zeros(len(embeddings), dtype=int)

    embeddings = np.asarray(embeddings)
    upper = min(max_speakers, len(embeddings) - 1)
    lower = min(min_speakers, upper)
    best = None
    for k in range(lower, upper + 1):
        try:
            model = AgglomerativeClustering(
                n_clusters=k, metric="cosine", linkage="average"
            )
        except TypeError:
            model = AgglomerativeClustering(
                n_clusters=k, affinity="cosine", linkage="average"
            )
        labels = model.fit_predict(embeddings)
        counts = np.bincount(labels)
        if len(counts) < 2 or counts.min() < 2:
            continue
        score = silhouette_score(embeddings, labels, metric="cosine")
        # Prefer a sensible silhouette, but penalize tiny singleton-heavy splits.
        score -= 0.015 * k
        if best is None or score > best[0]:
            best = (score, labels)

    if best is None:
        try:
            model = AgglomerativeClustering(
                n_clusters=lower, metric="cosine", linkage="average"
            )
        except TypeError:
            model = AgglomerativeClustering(
                n_clusters=lower, affinity="cosine", linkage="average"
            )
        return model.fit_predict(embeddings)
    return best[1]


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--input", required=True)
    parser.add_argument("--audio", required=True)
    parser.add_argument("--output", required=True)
    parser.add_argument("--json-output", required=True)
    parser.add_argument("--model", default="mlx-community/whisper-medium-mlx-4bit")
    parser.add_argument("--min-speakers", type=int, default=2)
    parser.add_argument("--max-speakers", type=int, default=6)
    args = parser.parse_args()

    input_path = Path(args.input)
    output_path = Path(args.output)
    json_path = Path(args.json_output)

    print("Transcribing with MLX Whisper...", flush=True)
    result = mlx_whisper.transcribe(
        str(input_path),
        path_or_hf_repo=args.model,
        verbose=True,
        word_timestamps=False,
    )

    segments = []
    for seg in result.get("segments", []):
        text = " ".join(seg.get("text", "").strip().split())
        if text:
            segments.append(
                {"start": float(seg["start"]), "end": float(seg["end"]), "text": text}
            )
            if len(segments) % 100 == 0:
                print(f"  transcribed {len(segments)} segments...", flush=True)

    language = result.get("language", "unknown")
    print(
        f"Transcribed {len(segments)} segments; detected language={language}.",
        flush=True,
    )

    print("Loading mono audio...", flush=True)
    audio, sr = sf.read(args.audio)
    if audio.ndim == 2:
        audio = audio.mean(axis=1)
    audio = audio.astype(np.float32)

    print("Loading speaker embedding model...", flush=True)
    classifier = EncoderClassifier.from_hparams(
        source="speechbrain/spkrec-ecapa-voxceleb",
        savedir="/tmp/corbanu_transcript/spkrec-ecapa-voxceleb",
        run_opts={"device": "cpu"},
    )

    emb_segments = []
    embeddings = []
    for i, seg in enumerate(segments):
        start = max(0.0, seg["start"] - 0.15)
        end = min(len(audio) / sr, seg["end"] + 0.15)
        if end - start < 1.0:
            mid = (start + end) / 2
            start = max(0.0, mid - 0.75)
            end = min(len(audio) / sr, mid + 0.75)
        sample = audio[int(start * sr) : int(end * sr)]
        if len(sample) < sr:
            continue
        with torch.no_grad():
            wav = torch.from_numpy(sample).unsqueeze(0)
            emb = classifier.encode_batch(wav).squeeze().cpu().numpy()
        norm = np.linalg.norm(emb)
        if norm > 0 and np.isfinite(norm):
            embeddings.append(emb / norm)
            emb_segments.append(i)
        if (i + 1) % 100 == 0:
            print(
                f"  embedded {i + 1}/{len(segments)} transcript segments...", flush=True
            )

    labels = cluster_labels(embeddings, args.min_speakers, args.max_speakers)

    raw_to_ordered = {}
    for idx, label in zip(emb_segments, labels):
        if int(label) not in raw_to_ordered:
            raw_to_ordered[int(label)] = len(raw_to_ordered) + 1
        segments[idx]["speaker"] = f"Speaker {raw_to_ordered[int(label)]}"

    # Fill any unembedded segments from nearest labeled neighbor.
    last = None
    for seg in segments:
        if "speaker" in seg:
            last = seg["speaker"]
        elif last:
            seg["speaker"] = last
    next_speaker = None
    for seg in reversed(segments):
        if "speaker" in seg:
            next_speaker = seg["speaker"]
        elif next_speaker:
            seg["speaker"] = next_speaker
    for seg in segments:
        seg.setdefault("speaker", "Speaker 1")

    turns = []
    for seg in segments:
        if (
            turns
            and turns[-1]["speaker"] == seg["speaker"]
            and seg["start"] - turns[-1]["end"] <= 1.4
        ):
            turns[-1]["end"] = seg["end"]
            turns[-1]["text"] += " " + seg["text"]
        else:
            turns.append(dict(seg))

    header = [
        "# Diarized Transcript",
        "",
        f"Source: {input_path.name}",
        f"Duration: {hms(len(audio) / sr)}",
        f"Language: {language}",
        "",
        "Note: Speaker labels are automated voice clusters. They are not identity-confirmed.",
        "",
    ]
    body = []
    for turn in turns:
        body.append(f"**[{hms(turn['start'])}] {turn['speaker']}:** {turn['text']}")
        body.append("")

    output_path.write_text("\n".join(header + body), encoding="utf-8")
    json_path.write_text(
        json.dumps({"segments": segments, "turns": turns}, indent=2), encoding="utf-8"
    )
    print(f"Wrote {output_path}", flush=True)
    print(f"Wrote {json_path}", flush=True)
    counts = {}
    for turn in turns:
        counts[turn["speaker"]] = counts.get(turn["speaker"], 0) + (
            turn["end"] - turn["start"]
        )
    print(
        "Speaker talk time:",
        ", ".join(f"{k}={hms(v)}" for k, v in sorted(counts.items())),
        flush=True,
    )


if __name__ == "__main__":
    main()

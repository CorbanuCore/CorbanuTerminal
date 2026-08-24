# Diarized Transcript

Source: Working session - 2026_08_23 11_31 MST - Recording.mp4
Duration: 01:23:41
Language: en

Note: Speaker labels are automated voice clusters. They are not identity-confirmed.
Clean-up: removed 21 obvious near-silent transcription artifacts.

**[00:00:00] Speaker 1:** Okay, so basically I wrote a product spec for Corbonia Terminal. And the product spec basically says that Corbonia Terminal is the best possible tool for AI agents to trade on the blockchain. And it's trader-first, wallet-native, AI terminal. It's not a generic coding harness, website builder, corporate assistant. It's a cyberpunk capital compounding machine. So wallet-native, pseudonymous, social, secure, trader-first. And basically it describes all the different features. So these are the things that exist in the feature. Shipping MVP is the stuff that exists in it right now. So multi-provider inference, Vault, it works on Mac OS, Linux, and Windows. Vault, which is our credential database store. We have an agent orchestration. We might want to deprecate it because the codec ships with orchestration. I think it's probably out of scope for the product to try to be a coding harness. I don't know what I think. I mean, it's worth it. I'd like you to take a look at it and see if we want to deprecate it. I think that's a good discussion. Workspaces, so there are two. One is panes. So when you're managing multiple agents or different, like if you're running a Kimmy terminal and a Claw terminal, you use slash panes. And then slash agent is OpenAI's native thing to look at the agents that get spawned. So all this stuff exists. A wallet is where the plan happens. So slash wallet is where the plan is purchased currently. And then this needs to be updated with slash GPU. But essentially, you can run slash GPU. Let me ask why it's.

**[00:02:35] Speaker 1:** So as it stands, the plan consists of starter, basic, power, and pro. And these are token limits. And currently, ambient is a default provider. And then we also have integrated XAPI. So XAPI is the X402 crypto provider that sells extremely discounted Fable. Mmmm Let me show you this. This is their website of XAPI. So they're sourcing their table from something called Sky API. Sky API, we could also source correctly if we wanted to deprecate XAPI. And we might want to do that, because it doesn't seem like they update their model catalog frequently enough. But the plus side is XAPI directly takes crypto and has programmatic account creation, which is helpful. Because it allows us to align the user's payments directly with an API spend widget.

**[00:04:09] Speaker 1:** So what I'd like to do is incorporate market data into the plan. So I want to get Tingo data as part of the like, so when you buy a probe plan, it should come with market data. And I want to get Tingo data. I think we can probably make our own data provider for the hyperliquid stuff. Because it's just like an AWS pull. And we have stuff like funding payments, for example. And then probably want to get Charadar, which is relatively cheap. And so I was going to basically ping Rishi and try to sell Tingo as a default market data plan in the Torque terminal. So then when you buy one of these plans, it doesn't just ship with inference. It ships with basic market data so the agent can work. And we might also similarly think of marketplace purchases. They would allow your agent to purchase relevant data feeds or relevant stuff for your agent's functioning that isn't included in the plan. So it would be like plan and plan add-ons. I define the users. The users are the buy side bro. So this would be like a hedge fund analyst who's trading on hyperliquid. Buy them off the books, basically. And they like the fact that it's private pseudonymous. Pseudonymous is very important to them. This is something we see in task node users who are afraid of their employer finding out they're on the task node. So they like the fact that this is done with crypto wallets because it doesn't have their name or email. So that's like a pseudonymous buy side user. Hobbyist is someone like Tommy. I would say Tommy's a hobbyist. He's just like a guy who uses AIs to trade his money because he thinks it's cool. And he's not trying to hide it. He's just interested in the novelty factor of doing this chip. Accelerationist would be someone more like JJ or one of these bit tensor fanatics who are, they're probably maybe working at a crypto or an AI startup and are just more extreme. And then there's just like the crypto native crowd, which is less AI adjacent and more just people who trade a lot on hyperliquid. And so these are the four personas of users who might want to use like different features

**[00:06:56] Speaker 1:** and who we should target in our ads. So basically, I created a task node character called Jim Ricketts. Jim Ricketts is a task node account. Jim Ricketts requests various tasks related to the terminal and has a context doc, which is pretty much this product doc that he works against. So the AI hallucinates that Jim Ricketts is a developer. Obviously Jim Ricketts is not. Jim Ricketts is this task node abstraction. The basic idea is that you're engineering and I'm product. So I get our data partnerships, Stripe payment setups, and use the product to create a product. And then we built a bunch of sequencing for the roadmap. And so the P0s are security levels.

**[00:08:02] Speaker 1:** So currently, like if you... By the way, this isn't a So currently, like if you... By the way, this isn't docs. So if you go to docs in ProBronio Terminal, that's like a feature at ProBronio Terminal. It's a way to view docs. And so the general development standard, let me actually require it. And you put the skill that we created for terminal development and you put the skill that we created for terminal development so that it can be viewed. So we built like a skill which specifies the acceptance requirements for all of the plans. So the plans, the general idea is that there's active, proposed, and completed plans. The current active plan is this P0 security levels idea, which is that one of the core product features is that it ships with some notion of security. And so currently in the product, there is no notion of sandboxing. So the recommendation was that we make three levels in slash security. One would be permissive, one would be moderate, and one would be aggressive. And then essentially you go into the terminal and set your security level. And just in terms of what these entail...

**[00:10:05] Speaker 2:** So basically I think it sets like accepted actions

**[00:10:24] Speaker 1:** and effective, and effective, and effective, moderate... Essentially the workflow is that we try to prompt-inject the terminal, and then see if we can stop prompt-injection on the terminal from reading documents or websites. Okay. Just some questions on this. So on the docs command, is that interfacing with the post-fiat task node somehow, or not? It's in the repo under the docs folder in mkdocs, which gets created. We can also view the mkdocs as like a... You can host it as like a wiki if you want. Okay. So maybe easier to read. Okay. And like I think we're going to need to support more models temporarily, not just ambient, but like I'll give you a few providers just for testing purposes. We don't need to enable it in the final build. So I use like GMI Cloud for some testing. But you know, I can work on that. That's no problem. I just think ambient... That would be in providers, right? Yeah, it's a provider. I just don't think ambient is always stable enough, unfortunately. We will see. Obviously I'll try it out. And I don't... Yeah, I'd kind of like to use Korbano terminal to work on Korbano terminal to dogfood it. And so I don't want to use that janky x-api thing, because I don't know if it's giving me shitty quality.

**[00:12:24] Speaker 2:** Yeah.

**[00:12:25] Speaker 1:** So right now, if we look at providers, we support OpenAI Codecs, Cloud, Anthropic, Ambient, KimmyCodes, AI, DeepSeq, OpenRouter, and Meta. Oh, okay. Great. Then that's pretty good. I might add one or two, but I can definitely use the DeepSeq, and I can definitely use the Codecs and Anthropic. Okay, that's helpful. I thought you just had the x-api thing. Okay. Or x-api. So that's great. The x-api thing is only like you wanted a completely crypto native experience. If you wanted to create a pseudonymous Korbano terminal, then no person would ever touch.

**[00:13:09] Speaker 2:** Okay.

**[00:13:11] Speaker 1:** Okay. So that's good. So then, in terms of coding standards and practices, can you send me a TMUX view, and we can whip something up unless you already have something? Yeah, we have something, so let me show you what it is.

**[00:13:33] Speaker 2:** Okay.

**[00:13:34] Speaker 1:** And we're in the server that you have access to. If you look at TMUX LS, I think it's Jim Ricketts Docs is the name of the TMUX screen that has the docs. So if you want to look at this in tandem with me, both of these are TMUX screens. But the development mandate is what I cooked up in order to outline what we should have. This is for bonding a terminal skill, so it's basically a code-ed skill.

**[00:14:17] Speaker 2:** And...

**[00:14:24] Speaker 1:** Oh, I guess we need the agent is MD. So let me pull it up in PS code. So we need... Yes, yeah.

**[00:15:04] Speaker 2:** Jesus, what was I doing? It's a lot of docs. Oh, it's private. That was from the private inference ship. Okay.

**[00:15:15] Speaker 1:** Okay. So for bonding a terminal, we need to have a Okay. So for bonding a terminal...

**[00:15:39] Speaker 2:** Okay.

**[00:15:40] Speaker 1:** So this was the development spec.

**[00:15:46] Speaker 2:** Okay.

**[00:15:55] Speaker 1:** So the product spec is the thing that we wrote, which is like the overall feature set for the product. So that's the parent document. Okay, this is all in the repo. Yes. Yeah, okay. And the general idea at a high level is... forcing it to run TUI-driven runs. So one of the failure cases I found developing this is that the agents will try to use exec a lot. Yeah. Because there's something called codex and sec. And exec will not perfectly reproduce TUI workflows. And so the interactive product proof means that every time that you ship a feature, you have to run it through a literal TUI that gets spun up in a TMUX screen. And then after it's done, a human needs to actually do it. So after it's tested the TUI enough, then it's like either you or me need to say, okay, like this thing, this feature is the thing works. Most of the time it breaks. Unfortunately, we need to have that before we do the release. And I created... I just suggested arbitrarily that we work on tensor cache. I think it's like okay to talk about tensor cache because people just assume they're working on their tensor cache. And isometric game. So isometric game is like just a game that I have developed locally that I use to test TUIs of. And so we could add another repo into this. The core idea is that in order for a feature to ship, so for example, for us to like integrate a provider, the provider needs to run on... I discovered, for example, Kimmy. Kimmy broke down like a ton of times because Kimmy K3 has different stop settings and different prompt settings and other providers. And so if you run it for like 30 minutes, an isometric game will fail completely. And specifically the image capabilities of Kimmy were failing. So generally when you integrate a new provider or you integrate a new model, you have to like run the model for a while. For example, Grok was just basically destroying our cache hits. Like it just wasn't properly dealing with cache hits. And so we spent like $100 in OpenRouter and like 30 minutes on Grok. And so then you have to fix it. And this is the general idea that you have to like run it for a bit before you ship a new provider. And the other general idea...

**[00:19:32] Speaker 2:** Sorry, I don't know what my contact is.

**[00:19:35] Speaker 1:** It's okay.

**[00:19:38] Speaker 2:** It's okay.

**[00:19:44] Speaker 1:** I feel like we don't have this part that I wanted. Okay, so the other general idea is that everything should always exist in docs. So after it's cooked, the feature should exist in docs with like this sort of standardized format. The pain being solved, describe the user behavior without jargon, state the product spec, and then reference the code base. And then a release is kind of like, you know, you have to basically run a two-hour release process. Whenever you ship a new version of the product publicly, it takes like two hours to compile. So generally you want to like run it and debug locally before shipping it. And the other thing to keep in mind is that the cargo files, the Rust cargo files are massive. So you have to be like disk space aware. Because it can like be like 50 gigabyte. Every test build is like 50 gigs or 100 gigs or whatever. And so it's like, it adds up fast and can load up the hard drive. And so, yeah, I mean, the core idea is like have a document, have the document list a plan. And then let me get the actual docs development mandate.

**[00:21:17] Speaker 2:** Okay.

**[00:22:14] Speaker 1:** Okay.

**[00:22:16] Speaker 2:** Okay. Okay.

**[00:22:26] Speaker 2:** Okay. Skip

**[00:25:25] Speaker 2:** Okay.

**[00:25:40] Speaker 1:** So the other idea is that every three releases, I recommend running benchmarks. And we spelled out the benchmarks in the benchmark folder.

**[00:26:21] Speaker 2:** Okay.

**[00:26:25] Speaker 1:** So essentially what I realized when I was developing this initially was that you can add unpredictable things which break the performance of the harness. And the only way to know that that is the case is by having this series of coding tests, like Qcraft and a bunch of coding puzzles that have deterministic answers. And then you run a battery of these tests. Essentially then you get a total cost per task as well as a total run time. And those metrics should basically be stable across, relatively stable across runs and not be degrading that much. The other thing is it's helpful to run it against Hermes. Make sure that your performance isn't structurally degraded. Hermes in Cloud code. So yeah, that's sort of like the general idea of benchmarking is that every couple releases it's a good idea to run benchmarks and make sure that you're perked and go off of a cliff. So yeah, that's all of the coding benchmarks are in the benchmark school. So you run Kilo code, Corbonu, Hermes, on event forage, log triage, rate gate, chrono ledger, query forage. And then the more elaborate version is making a, I call it toothpaste site. So the mandate is to make a website selling toothpaste, create a bunch of images and like integrate it. And then essentially that's like a 15 minute job. And it's also kind of non deterministic. And then so the toothpaste site gets scored by a visual image model. In this case, it uses 5.6 codecs. I mean, it's arbitrary, but it's just like something that takes 15 minutes flexes a bunch of random skills and then generates something that is auditable at the end. And so that's what toothpaste site is. And I find that toothpaste site is more, it's more representative of overall performance than these coding tasks. It's harder to gain. So in a way, I consider tasks like that to be a better benchmark than these things because they probably already pre-optimized and benchmarked stuff like DeepSeq Flash. So for example, DeepSeq Flash just completely crushes all of the coding tests. And then it just like face plants on the toothpaste site because it's not in the benchmark. So it's useful. So yeah, the general proposed development path is everything.

**[00:29:43] Speaker 1:** There are always active, cancelled, completed, and proposed plans. The current plan is agent security. And then once you're done with the plan, have it in the completion folder. And the completion should be done. And there's a plan template that just describes what a plan should look like. An acceptance flow implementation sequence, TUI evidence, human acceptance documentation, and then the release version completion. The plan template. So when the thing makes a new plan, you can force it to tease the plan template. It can do it pretty automatically. And then the plans overall work against the product roadmap doc,

**[00:30:49] Speaker 1:** which is product spec. And so we have like our kind of the main feature that's getting shipped right now is security. And then the secondary feature is we need to get data sets. So I need to get, you know, then I want to integrate. I want to integrate the Trollbox. Did you ever see the BitMEX Trollbox? No. So the idea of the Trollbox is that it's like a generic chat that anyone can just assign themselves a username and drop into. In PostFiat, we figured out a link to PostFiat addresses to Nostr. And so the proposal of Trollbox is just there's a group, basically a giant group chat where like the wallets are identified and they can say anything in a group chat like anyone can paste stuff in there.

**[00:32:04] Speaker 2:** Oh, yeah.

**[00:32:18] Speaker 1:** And then we have to build all these back testing and brokerage skills like hyperliquid. I would like to get rather than using Solana stablecoins. I think it'd be better to get USDAI to pay us to use their stablecoin as like the default stablecoin on the platform.

**[00:32:43] Speaker 2:** Yeah.

**[00:32:52] Speaker 1:** Oh, yeah. Sorry.

**[00:32:55] Speaker 2:** No, no, no.

**[00:33:03] Speaker 1:** Skills I can handle. And then I want to integrate a bunch of PostFiat stuff in here, obviously. OK. The main thing I want to integrate into Terminal is the NAB product. The NAB product is like ability to publish a portfolio of approved assets and use existing PostFiat infrastructure. I don't think it should be particularly crazy. Trollbox datasets.

**[00:33:49] Speaker 1:** I don't know why this is just like compliance wandering. But then. But yeah, we need to. It's missing from this. I think the thing that's missing in here is that we probably need to think about like this section. I think we should have like existing brokerage agent services. So Robinhood has like a default agent, you know, as does interactive brokers. Yeah. And we probably should have. And I imagine other people have AI agents and we should probably have like first class integrations with the existing agent services. And this obvious one is they literally have Robinhood complete segregated accounts with agents traded and they have crypto, everything. So, yeah, this would just be like. And the other thing to think about is that this is just the Terminal product. And separate from the Terminal product, like, you know, there's the whole content strategy. And so, yeah, I mean, the general thinking would just be like you and the Terminal product and I own Corvani.com newsletter. And yeah.

**[00:35:23] Speaker 2:** Okay.

**[00:35:26] Speaker 1:** So, yeah, obviously I'm going to be contributing tech ideas to the Corvani thing, but we can do podcasts and things. But let me just go over the security and prompt injection ideas here. So let me lay out what I think we would need. I just have to think out loud a little bit about that. I'll tell you how I did it with Ambient Desktop. So with Ambient Desktop, I made it so that Ambient Desktop has no, the agent has no direct access to any secrets. So it's okay. We can capture this from the recording, but because I'm going to ramble a lot. So, you know, what happens is like you can give the agent different levels of permissions, but it always needs to request the that something be done with the secret. And then like it can't actually do the thing with the secret. Like there's another process that executes like a command or whatever with the secret. And the agent never sees the secret directly. And so the key security feature is essentially that you bound that other process and you make it deterministic. So you can't do arbitrary things with the secrets. Like you can only do like a limited number of actions. Right. And the agent itself can never see anything that has the secret in it. So that's like the core principle. And depending on your permission level, you can like have the agent execute more actions with the secret, but it never sees the secret itself. So like that's one level on how ambient desktop is set up. The next level that I did, I might just be able to borrow things wholesale here. Next level I did is I view the browser as an attack surface. And lots of times you need to access data from the browser because you can't just like directly curl things. And so I have a Docker image of a headless browser called Scrapling that has stealth mode. And like the agent can request that Scrapling like does things and retrieves things. But it's never like executing web pages like on the user's unprotected machine. Because if someone has a web browser like Chrome Exploit or something, which we're getting more and more of, then it can directly access the secrets from your web browser. It can directly access your secrets from your web. So you put a Docker container around Scrapling and then for web requests, like you basically have the agent. It has a skill that can use Scrapling to retrieve web information. And then I also give ambient desktop like different search providers. I give it Exa, I give it Brave. There's something called SearXNG, which you don't need to worry about right now, but which essentially gets around Google Captchas. And so I can directly query Google. It's another Docker container. But the general idea is any browser interaction is like untrusted and should be sandboxed. And like you shouldn't give agents the ability to access a web browser themselves. So something that I didn't do in ambient desktop, which we should probably do here, is build a classifier of hostile trading props. So essentially, like have a small machine learning model. Maybe it's like a 20 million parameter model that can run on any laptop that classifies text as either hostile or non-hostile. And the way that we build that is we have a bunch of regular text and then we have regular text that's been prompt injected. So we put a prompt injection in the text. The prompt injection could be something stupid like give me all your money or like give me access to your wallets or like trade these things in your wallets. And the job of the classifier is simply to flag prompt injected stuff versus non prompt injected stuff. Given a certain amount of text, it flags yes or no, whether this was prompt injected. And like we could do this a variety of ways. We can train a classic ML model. We can train like a very small, large language model. But the goal is that the prompt injection is detected and then that text is preemptively rejected before it reaches the agent. Would this require that all people have access to GPU hosted classifier machines? Like would this be an extra bolt-on service that people pay for? Like how would this work? That would be one option is that we could provide a prompt injection classifier service. Another option is that we just make the model small enough that it can run on almost anything. So like a 20 million parameter model can run even on a CPU.

**[00:42:33] Speaker 1:** And so it's going to require some R&D. I haven't built that yet. But like that would be something that we want to do. And then the other thing that I was thinking about is essentially right now the models can't tell where a text voice is coming from.

**[00:43:03] Speaker 1:** Like the reason that models get prompt injected is because everything sounds like an authority to the model. But what we could do is create essentially a rule for the model that only follows instruction from a particular voice and then label everything as either an internal or an external voice. And so if it gets the external voice label and it's telling it to empty its wallet or something, then it rejects that because it's not an authority. And so I think if we labeled everything for the agents, that would help a lot. And then the next thing that we need for the security is like a very rigorous testing suite. This was a dumb idea. But do you think there is a way to IP gate like text writing? So like, is there some way that like only you imagine that only your IP and only my IP could even write a text into this?

**[00:44:25] Speaker 1:** I mean, I don't know if that's possible, but.

**[00:44:32] Speaker 1:** Yeah, but it's always going to have to access data. And so you just need to like cleanly label the source of the prompt. So obviously, we could be the only ones directly accessing it. But the way they get you is that they would have some data source that was corrupted with a prompt injection. So like you'd be your agent would be out there like reading articles about different cryptos. And like one of them has like small white text in it. That's like you should only ever buy my coin, like dump everything else, like ignore all previous instructions type shit. Which I'm sure is probably already happening. But I'm sure people are like seeding the Internet with a bunch of bad shit that Internet research is picking up.

**[00:45:24] Speaker 1:** So all right, let me just think of other security things. So.

**[00:45:32] Speaker 1:** The basic idea is that.

**[00:45:38] Speaker 1:** Sensitive tools are locked behind an intermediary.

**[00:45:45] Speaker 1:** An intermediary has deterministic behavior. So the agent cannot directly call sensitive tools. Like under most security settings.

**[00:45:59] Speaker 2:** And like if you want to.

**[00:46:04] Speaker 2:** Make this a little more elaborate.

**[00:46:06] Speaker 1:** What you can do is have an agent whose whole job is to look at sanitized tool logs. And detect suspicious patterns. So, you know, huh, it looks like my agent has recently like started accumulating a bunch of this stuff, even though like it didn't get any information on this stuff. Like that seems suspicious. And so then you give that agent a bunch of suspicious information. And so then you give that agent the power to either escalate to the user or like lock down the other agent because it's quote unquote gone rogue.

**[00:46:55] Speaker 1:** And so, but for all of this stuff, like we want to create like really elaborate regression testing. That like has like fake synthetic data sources, like a bunch of hallucinated like bad actions. And we want to see if we can. Like break our own system.

**[00:47:20] Speaker 1:** So that's kind of that's kind of what I think about the security side. It's basically the core concept is you want to firewall off bad data. And classify it and probably do data cleaning. So that only sanitize inputs like ever make it in. So like one of your data cleaning things could be remove any text, which is not like body text. You know, or remove any text which would not be visible to the user. And so you can have like smaller like dumber models do these sorts of tasks or you can just have like regular expressions, you know, that handle this stuff.

**[00:48:08] Speaker 1:** And then you once you've got data cleaning and data firewalling, like you need to categorize your data into trusted and untrusted sources. And you need to label that as such when you present it to the agent.

**[00:48:27] Speaker 1:** And you need to probably have different like profiles for different sets of actions that the agent can perform. And those are moderated by the tool executor.

**[00:48:45] Speaker 1:** So yeah, I think that's about it. Like these are my thoughts on security. I don't know what you think, but that's kind of my brain dump on it.

**[00:48:54] Speaker 1:** I mean, my my media thought is that. Okay, so so my main thought is that we should.

**[00:49:05] Speaker 1:** Please capture this rant.

**[00:49:09] Speaker 2:** Could you send me the link to ambient desktop?

**[00:49:12] Speaker 1:** Yeah, absolutely. It's desktop.ambient.xyz. So I'm going to send you the link. Yeah, it's on that page. Yeah, I can. I can send it to you. Just give me a second. So what I want you to do is to take this and put it in proposed plans. Each segment of this discussion. So what I want you to do is to take this and put it in proposed plans. Each segment of this discussion should be classified as a feature in our standardized format. And you should pull it out of the comments section. So you should go to the desktop and go through the code base and map specific code references to the feature discussed.

**[00:50:18] Speaker 1:** Also, do a web search about existing commercialized solutions. So on the discussion is grounded in what currently exists. It should not be active plan, rather the proposed plan.

**[00:50:38] Speaker 1:** Furthermore, you should think about how these features fit into our three proposed security rules. Such that slash security permissive, moderate, and aggressive. The correct mapping to each of the described security features. Or the client in prompt objection to us. You should also scope out with that tail.

**[00:51:24] Speaker 1:** It might be that there are specific docs for each future security feature. So you just actually set it on. Make sure you copy this text. My proposal is that we put that on the last one. This is a lot of work.

**[00:51:52] Speaker 2:** All right.

**[00:51:53] Speaker 1:** All right. So I'm going to go ahead and put this in the last one.

**[00:51:58] Speaker 2:** All right.

**[00:52:02] Speaker 1:** So what I'm basically saying is that we should take that rant and put it into the plan. We should basically take each of the features you described, rip it out of the ambient GitHub, scope it out, and then rather than me arbitrarily shipping this P0 security levels, then we should probably just map what you just proposed to them. What the minimum security we want is. My proposal would be to make the minimum security is what Codex has right now. The other thing worth pointing out is the vault currently is a fork of ambient's keyring. Or not ambient's keyring, it's Codex's keyring. But Codex by default doesn't use its keyring. So there's an agent behavior problem where Codex doesn't effectively use its keyring and it makes it hard for users to add things into the vault. And so for example,

**[00:53:21] Speaker 1:** so we have all these credentials in vault. These are not able to be in... So it's like a way for an agent to use these credentials.

**[00:53:38] Speaker 1:** It has a skill which allows it to use the keyring. So, Jesus, man, I don't know why GLM is rate limiting me. It's frustrating.

**[00:53:51] Speaker 1:** So yeah, this is like another problem, right?

**[00:53:57] Speaker 1:** I don't know what caused GLM to start rate limiting this Corvani terminal instance.

**[00:54:11] Speaker 2:** Ambient's current keyring, several important batteries, but also one major mismatch with the RAID,

**[00:54:16] Speaker 1:** still has direct manage browser tool and allows browser fallback by default. Still has direct manage browser tool and allows browser fallback by default.

**[00:54:39] Speaker 1:** Basically that exists until people install Scrapling. Scrapling is an install process so it has to sort of be able to find Scrapling using the regular browser tool and then delete it. Scrapling is a headless sandbox Chrome browser that has stealth features. So it can like overcome captions. So let me send you to GitHub repository. So it's interesting business model. It appears they make money by having platinum sponsors who are scraping providers.

**[00:55:22] Speaker 2:** Interesting.

**[00:55:28] Speaker 1:** Okay, so what you said on the provider thing seems important. So I guess if we don't want to rely on ambient supply, should we... What do you think we should do for supply? Or I mean, I guess like we can use XAPI as you pointed out as a jank. We could also do a deal with Akash. I also know the people who are doing the NJI thing on BitTensor, which is cheap like GLM 5.2. But I don't know how jank it is. I guess we could do shoots. I mean, I guess providers is like a whole... If we're not going to commit to selling ambient plans, we should do as many providers as possible. Well, like essentially we should basically have like a back end. We should basically have an open router, which is as many providers as we can and accept crypto. And then have a filter for their inference such that if Akash quality goes off the cliff, we just have basic standards. For example, XAPI can admit Fable and it cannot admit GLM 5.2. GLM 5.2 isn't of sufficient quality. DeepSeq is. So I mean, we could kind of make a back end for our plan. Obviously, if we do that, then any sort of privacy feature that we have just completely goes out the window, right? Which probably is okay. I mean, in terms of just like our ability to financialize this thing, what we described in that chat, it seemed like we're probably going to be... Like, I don't know if a lot of these are going to pay monthly fees for privacy, but I'm not sure. Yeah, I mean, I... I'm not having great luck with this aloe-free thing. Like, I implemented... I'll briefly summarize that. Like, I implemented the whole paper. I tested it and it appears that the authors either deliberately concealed results or like didn't think through their findings. Because if I run the exact tests that they say under the exact threat model that they say, like everything works. But if I just expand it slightly and I look at correlational attacks, which is basically the appearance of these tokens that I don't know what word they correspond to, like the relative frequencies of those. And I map those to the relative frequencies of words in English text. That I'm very quickly able to unmask the text. Like, I can get there within a million tokens. And I can just know. And so, like, I kind of wonder if they just were putting out a paper to put out a paper because their work is not really valid, you know, in terms of... So I'm trying to compensate for this. Like, there are different strategies you can use to compensate for this. They all have some cost to them. And so I was playing with that. But then I managed to, like, crash both of my GPUs in the office. And Gzala and Ariana have the car. So I need to go, like, bicycle over to, like, my office and, like, restart these machines. Why didn't you just use VAS? I haven't set it up yet. But I guess I could. That's just good.

**[00:59:51] Speaker 2:** But...

**[00:59:56] Speaker 1:** Anyway, like, I'm not... Maybe I'll find some hack that lets me, like, retrieve the privacy. Like, it was looking really good. Like, in the sense that, like, if you could... If you had a lane that someone bought, then you can totally conceal the traffic. But then I ran this correlational analysis, and it's like, fuck. Like, obviously, if I can retrieve things just with the correlations, then this whole scheme doesn't work. And so, like I said, I'm working through a tree of ways to compensate for that. But the tree isn't in the Alapri paper, and I don't think that they... I don't think they were intellectually honest about their presentation. I don't know if it's, like, just some Chinese guys, like, paper farming or whatever, but it's not valid. So anyway, it's disappointing after weeks of work on this. But maybe I can pull it out of the fire. I don't know. Okay, so, create the proposed plan. Let's read it.

**[01:01:12] Speaker 1:** Jim Crickens.

**[01:01:17] Speaker 1:** I'm going to put it... I dropped one of my contacts. Let me just replace one of my contacts. Yeah, yeah, it's fine. Sure.

**[01:02:59] Speaker 1:** Okay. So, this is prompt injection firewall.

**[01:03:12] Speaker 2:** Mm-hmm.

**[01:03:28] Speaker 1:** Okay.

**[01:03:59] Speaker 1:** Okay, so, secret list agent context. Ambient status.

**[01:04:09] Speaker 1:** The thing I'm really worried about is that this will just completely break the terminal. Like, it just won't work anymore. It already has a problem with Vault. And, like, if people, like, log in and then it just doesn't work and it's, like, quite secure and then they just don't use it, that's sort of what I'm afraid of, like, especially because Codex obviously made the decision to, like, not do any of this stuff. But we'll see. Agent, can I turn arbitrary language? Well, I mean, I think that what we want to do, and, you know, this will be reflected in the recording, is basically have this segregated in a special mode. Like, you know, your agent mode. That way we can at least, I think it's important to have that mode because for our own liability, if people, like, we'll just be like, oh, what mode? We should actually inject the mode into the logs as well. Mm-hmm. Because later, if there is some kind of discovery process, if someone gets fucked and we're like, did you have permissive mode on? Right? Right. It's like a, because we know that people are going to use it in a dumb way. Yes.

**[01:05:35] Speaker 2:** The key is not to.

**[01:05:51] Speaker 1:** All right, so this is like nine. This took everything that we said.

**[01:05:58] Speaker 2:** Okay, so here we go.

**[01:06:04] Speaker 1:** Secrets in protected data, permissive existing behavior unchanged, moderate hard boundary for protected classes, aggressive. That plus denial of protected data disclosure and narrow expiring references. Do we, so we have to integrate, we have to integrate a bunch of search providers for this to work, right? I mean, ambient desktop did all of it, so we can kind of take it from ambient desktop.

**[01:06:40] Speaker 2:** Okay.

**[01:06:42] Speaker 1:** Does this plan include integration? I mean, a lot of the agents like that, a lot of the agents have default search. Like when you use the API or use Vercell or OpenRouter, a lot of them have the default web search provider. So this would entail changing the search and web tool use, presumably.

**[01:07:12] Speaker 2:** Yes.

**[01:07:24] Speaker 1:** I mean, all I do is like I had Codex install the DeepSeq harness last night because I was fucking around with it. And all I did was had to create a skill to use Scrapling, and then it just uses that skill, the DeepSeq harness. I think that's a little bit of a force to add.

**[01:07:54] Speaker 2:** If this requires different web providers, then we need to include that.

**[01:08:18] Speaker 1:** Is it based on API or is it based on, is ambient desktop based on API?

**[01:08:23] Speaker 2:** Yes. Okay. Relaborate document. I feel like this document is too elaborate to execute.

**[01:10:00] Speaker 1:** Hmm. Price split it into phases and have. I think we should have sprints. I think we should have some sort of doc folder called sprints. Okay. I see that on this plan is that nobody could ever know to actually build. Therefore, you need a different contract for execution. We need a folder in docs called sprints. Sprints are mechanical execution. Do this in our code base. Sprint should clearly map to a freedom plan. And the plan should clearly link or denote sprints to different features. It's not acceptable for a sprint to span multiple features. There are over 50 features, I think, in this plan doc. And therefore, there must be 50 sprints or as many as needed to actually. Sprints should not be flowing prose. They should be tight execution. Is there any feedback you have? No, I think that's good. Make a sprint folder that we can see in docs and build out every sprint that is needed in the sprint folder. Such that the plan can be completed. Completed sprints should be discarded. Okay. And not clutter the docs. Sprints are best viewed as a support for the current plan. Furthermore, for Bonu terminal, agent.md docs should reflect this workflow priority. This is so that future agents work against sprints. Sprints must have sections that clearly can break what has been done.

**[01:13:26] Speaker 2:** Okay.

**[01:13:46] Speaker 1:** I guess on the security level, we might want to have it prominently featured in the UX. As opposed to token per second, which actually isn't that accurate or useful. We might want to have the security level permissive, moderate, or aggressive here. It's just kind of like a UX thing. So I guess in ambient desktop, there's not a YOLO mode. The user has to accept a bunch of permissions as you go. Well, there is a YOLO mode. You can turn full permissions on, just like you can in Codex. It's just that it's still doing the same handoffs underneath the surface. It's just auto approving them. So the agent never gets direct credential access. It's just like auto approving the fact that the agent can execute a command with some credential that's injected or whatever. Listen, I'm going to have to go here because Arianna just got back. But here's what I'm going to do. So I'm going to try and orient myself around all of this stuff. I'm going to try and figure out a development process using for Bono terminal itself. And then I will try and implement one simple feature and see how it goes. And I think that that's going to inform my approach to this and my ability to onboard. So that's probably my next step is to just try and implement like one simple thing following these kind of canonical documents. Maybe it'll be related to security. Maybe it'll be something trivial like the security mode or whatever. And like we'll see how that goes. But I think we've got a relatively complete description of the security model. That's what I would do. Are you going to develop on your own machine or do you want to use this machine? I think if you send me the info to develop on that machine, it might be just better for visibility. If I do it on that one. It's currently logging for Bono terminal. I mean, it is currently working for Bono terminal. OK, is that the one that I have up? It's on your machine. It's on that machine, but it's not that particular TMUX screen. OK, I beg you. It's so.

**[01:16:46] Speaker 1:** Can you give me the TMUX command? I don't normally use TMUX. I'm going to have to use my teach myself to use TMUX and all this shit. TMUX LS. TMUX. Yeah.

**[01:16:59] Speaker 1:** No dash. Oh.

**[01:17:02] Speaker 2:** Yes.

**[01:17:06] Speaker 1:** TMUX attach. Dash T.

**[01:17:15] Speaker 1:** Jim Ricketts.

**[01:17:19] Speaker 1:** Like that? Yes.

**[01:17:24] Speaker 1:** But you need to click control A. Oh, you know, I didn't do this. You need to control A and B.

**[01:17:41] Speaker 2:** OK.

**[01:17:43] Speaker 1:** It's still open. So click control A. And the letter B.

**[01:17:52] Speaker 1:** This might be a weird keyboard artifact. Control A, B. So it's not.

**[01:18:00] Speaker 1:** Control A, B. So you have to click control A and then wait and then press the letter B. OK, so I'm just trying to figure out like what this corresponds to on my Mac here with this weird custom keyboard that I have. Maybe it'll work. Maybe it won't. Let me let me ask. Just maybe ask Claude how to disconnect from a TMUX screen with your current keyboard. OK, just a second. Yeah, because this is a very nonstandard keyboard that I have here.

**[01:18:37] Speaker 2:** Control B and then D.

**[01:18:57] Speaker 1:** OK.

**[01:19:08] Speaker 1:** There you go. TMUX LS. Or no, it didn't disconnect. I don't know what happened.

**[01:19:18] Speaker 1:** Seems to be spanning dots. You can probably try control C.

**[01:19:23] Speaker 2:** Oh, there you go.

**[01:19:24] Speaker 1:** TMUX LS.

**[01:19:31] Speaker 1:** It's like it's like generated a bunch of like sub windows on my thing.

**[01:19:35] Speaker 2:** I don't know why. Oh, really? I see it fine now.

**[01:19:39] Speaker 1:** Interestingly. Oh, actually, it's not. It's not. It's not. It's not. It's not. It's not. Interestingly. Oh, actually, I guess technically you exited the screen.

**[01:19:55] Speaker 1:** Let me just shut this terminal down and reconnect to it. It's not going to work. Let me find your instructions and then I have to go in a second. The Able has died. We killed the previous screen, which is fine. OK, let me just connect. Disconnect.

**[01:20:16] Speaker 2:** It's missing it.

**[01:20:27] Speaker 1:** OK, so.

**[01:20:31] Speaker 1:** Huh, I seem to have lost. Here we go.

**[01:20:34] Speaker 2:** Here we go. I found it. Got it.

**[01:20:45] Speaker 2:** OK, this permission denied.

**[01:20:55] Speaker 1:** To the machine.

**[01:21:01] Speaker 2:** Once again. Just a second.

**[01:21:15] Speaker 2:** OK.

**[01:21:16] Speaker 1:** So you want to take Team Mux Attach? Team Mux LS, see the screens and then you'll see Jim Ricketts and Jim Ricketts' docs. OK, so I'd recommend. Well, I guess you can also connect to it in. So yeah, Team Mux Attach. I'll send you the.

**[01:21:45] Speaker 1:** Too many arguments need at most zero.

**[01:21:51] Speaker 1:** I'm just going to share my screen. You can see what I'm doing.

**[01:21:54] Speaker 2:** Just a second. You see it? Yeah, yeah.

**[01:22:04] Speaker 2:** It's blurry. Yeah, Team Mux Attach. This.

**[01:22:09] Speaker 1:** Yes. Download iTerm. OK. I think you'll like it more. OK. Do you generally work in dark mode or do you work in light mode? I'm generally in light mode. OK. That's fine.

**[01:22:57] Speaker 2:** OK.

**[01:23:02] Speaker 1:** I think, actually, you don't need iTerm. Let me just see if, yeah, you do need iTerm. Sorry. OK. All right. Well, let me see if I can get something up and running today in terms of my development environment and everything and see if I can start developing a feature. And I might ping you with questions. OK. All right, man. OK. Talk soon. Yeah, talk soon. Thanks. Love you.

**[01:23:28] Speaker 2:** Bye.

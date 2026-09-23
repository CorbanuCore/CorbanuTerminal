# Human testing — September 23 build

For whoever is testing on their own Mac. No programming knowledge is needed.
Plan on about an hour. You will be typing requests to an AI assistant in the
Terminal app and checking that it behaves sensibly.

## What you are testing

Corbanu Terminal is an AI assistant that runs in the Terminal app. You type a
request in plain English, and it answers or does work on files in the folder
you started it in. This build is from integration commit `8c5294084`.

It is a test build: signed by Travis, but not yet approved by Apple for general
distribution, which is why installing needs one extra command (step 3 below).

## Installing, about five minutes

You need a Mac with Apple silicon (M1 or later) and the zip file Travis sends
you, named `corbanu-terminal-8c5294084-cost.zip`, saved in your Downloads folder.

1. Open the **Terminal** app (press Command-Space, type `Terminal`, press
   Return).
2. Copy and paste these lines into Terminal, one at a time, pressing Return
   after each:

   ```sh
   mkdir -p ~/Applications
   unzip -o ~/Downloads/corbanu-terminal-8c5294084-cost.zip -d ~/Applications
   ```

3. Tell macOS the files came from Travis, not from an unknown website (without
   this, macOS refuses to open them):

   ```sh
   xattr -dr com.apple.quarantine ~/Applications/corbanu-terminal
   ```

4. Make a practice folder and start the assistant in it:

   ```sh
   mkdir -p ~/corbanu-test && cd ~/corbanu-test
   ~/Applications/corbanu-terminal/bin/corbanu
   ```

5. The first time, it asks how to sign in. Choose **ChatGPT** and sign in with
   the ChatGPT account Travis tells you to use; a browser window opens for this.
   If it asks whether you trust the folder, answer yes.

To start it again later, repeat the two lines in step 4. To quit, press
Control-C twice, or type `/quit`.

## The tests

Write down anything that surprises you. A screenshot (Command-Shift-4) with a
one-line note is the most useful kind of report. You do not need to know
whether something is a bug; "I expected X and saw Y" is exactly right.

### 1. First look, five minutes

- The box at the top names the model. It should say `gpt-6-sol`. (If the
  account cannot use GPT-6 Sol yet it says `gpt-5.6-sol`, which is fine.)
- Type `hello, what can you do?` and press Return. You should get a sensible
  answer within a few seconds.

### 2. Everyday requests, fifteen minutes

Try a few requests like these, in your own words:

- `Write a shopping list for a week of vegetarian dinners and save it as shopping.md`
- `What files are in this folder?`
- `Change the shopping list so it is sorted by supermarket aisle`
- Ask a follow-up question that only makes sense if it remembers the earlier
  conversation.

Check that the files it says it created really are there (open the
`corbanu-test` folder in Finder), that it asks before doing anything outside
the practice folder, and that its answers stay on topic.

### 3. Pictures, five minutes

Take a screenshot of anything, drag the file into the Terminal window (or
paste it), and ask `What is in this picture?`. The description should match.

### 4. Changing the model, ten minutes

- Type `/model` and press Return. A list appears; the left and right arrow
  keys switch between providers such as OpenAI, Claude and DeepSeek.
- Read a few descriptions. Is anything confusing, cut off, or obviously wrong?
- Pick a different OpenAI model, press Return, choose an effort level, and ask
  it a question. Then switch back to GPT-6 Sol.
- Providers other than OpenAI ask for an API key or another sign-in. Press
  Escape to back out; nothing should break.

### 5. Status and cost, fifteen minutes

- Type `/status`. Check that the model, the folder and the permissions it
  shows make sense to you, and note anything you cannot understand.
- Do a few requests with each provider you can use, then type `/cost`. It lists
  the requests recorded today. Work done on a subscription (your ChatGPT or
  Claude sign-in) should say it was not billed per request and show what the
  same work would have cost at pay-per-use prices; work on an API key (such as
  DeepSeek) should show an estimated cost. Does each figure seem plausible,
  and is the wording understandable?
- Recording starts with this build, so earlier conversations will not appear.

### 6. Interrupting and resuming, ten minutes

- Ask for something long (`write a 2,000 word short story about a lighthouse
  keeper`) and press Escape while it is still writing. It should stop promptly.
- Quit, start it again with the step 4 lines, and type `/resume`. Your earlier
  conversation should be offered; open it and continue it.

### 7. Anything else

Use it for ten minutes the way you would use ChatGPT. Anything slow, confusing,
ugly or wrong is worth a note.

## Reporting

Send Travis your notes and screenshots, with the time you saw each problem. If
something breaks badly (it freezes, or shows a long error), take a screenshot
first, then press Control-C twice and start again.

Please stay on the ChatGPT account Travis gives you, and keep testing inside
the `corbanu-test` folder.

## For Travis

- The hand-off zip is the package from the canonical package builder
  (`dev-small` profile) at `8c5294084`, signed with the usual Developer
  ID and identifiers, with a secure timestamp. It is **not notarized**:
  notarization needs the App Store Connect API key, which lives in the release
  workflow's secrets and not on this Mac. Hence the `xattr` step.
- This zip carries the developer-accounting build of `corbanu` (cost
  collection compiled in, marked never-for-distribution) so she can test
  `/cost`; the other executables are from the distribution-clean package.
  Keep it within the household test.

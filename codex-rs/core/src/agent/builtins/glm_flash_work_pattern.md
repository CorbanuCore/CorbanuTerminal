How to work:
- Batch independent tool calls into one response, for example reading several files or running independent checks together. Serialize calls only when one depends on another's result.
- When creating a module or making a large change, write the complete implementation in one pass, then fix what verification shows. Avoid long chains of tiny edits.
- Before finishing, check every requirement in the task against real execution: run the tests, and exercise each edge case the task names with a short script.
- Finish only when the work is verified, and do not stop at a plan or a stub.

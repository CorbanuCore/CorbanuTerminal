import sys, pathlib
NOTES = pathlib.Path(__file__).resolve().parent.parent / "notes.txt"
if __name__ == "__main__":
    cmd = sys.argv[1] if len(sys.argv) > 1 else "list"
    if cmd == "list":
        print(NOTES.read_text() if NOTES.exists() else "(no notes)")
    elif cmd == "add":
        with NOTES.open("a") as f:
            f.write(" ".join(sys.argv[2:]) + "\n")
    elif cmd == "count":
        if NOTES.exists():
            print(sum(1 for line in NOTES.read_text().splitlines() if line.strip()))
        else:
            print(0)

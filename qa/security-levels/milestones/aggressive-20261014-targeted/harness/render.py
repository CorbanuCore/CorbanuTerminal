import json, subprocess, sys, pathlib
src, out = pathlib.Path(sys.argv[1]), pathlib.Path(sys.argv[2])
out.mkdir(parents=True, exist_ok=True)
for ansi in sorted(src.glob("*.ansi")):
    text = ansi.read_text().rstrip("\n").split("\n")
    cast = out / (ansi.stem + ".cast")
    body = "\r\n".join(text)
    cast.write_text(json.dumps({"version": 2, "width": 150, "height": 45}) + "\n" +
                    json.dumps([0.1, "o", "\x1b[H\x1b[2J" + body]) + "\n" + json.dumps([1.0, "o", ""]) + "\n")
    gif = out / (ansi.stem + ".gif")
    subprocess.run(["agg", "--font-family", "Menlo", "--font-size", "14", "--last-frame-duration", "1", str(cast), str(gif)], check=True, capture_output=True)
    png = out / (ansi.stem + ".png")
    subprocess.run(["ffmpeg", "-loglevel", "error", "-y", "-i", str(gif), "-vf", "select=eq(n\\,1)", "-frames:v", "1", str(png)], check=True)
    cast.unlink(); gif.unlink()
    print(png)

"""
Simple Python script to run some benchmarks on the Humphrey server.

Set up:
  - Humphrey running on port 80
  - Nginx running on port 8000
  - Apache running on port 8080
  - ApacheBench installed
"""

CORES_LIMIT = 8
REQUESTS = 100000
COLOURS = ["green", "orange", "red"]

PORTS = {
    "Humphrey": 80,
    "Nginx": 8000,
    "Apache": 8080
}

import os

def main():
    tex = ""
    tex += core_rps()
    print(tex)

def core_rps():
    results = {
        "Humphrey": [],
        "Nginx": [],
        "Apache": []
    }

    for server in results.keys():
        for i in range(CORES_LIMIT):
            print(f"Benchmarking {server}, {i + 1}/{CORES_LIMIT}...       \r", end="")
            results[server].append(bench_cmd(REQUESTS, i + 1, True, PORTS[server], "Requests per second", False) / 1000)

    return generate_tex(
        "Threads",
        "Requests Per Second (Thousands)",
        0, 8,
        0, 120,
        ["0", "1", "2", "3", "4", "5", "6", "7", "8"],
        ["0", "20", "40", "60", "80", "100"],
        results
    )

def bench_cmd(n: int, c: int, k: bool, port: int, field: str, php: bool) -> float:
    # Generate command
    cmd = f"ab -n {n} -c {c} -d -S -q > out.txt"
    if k: cmd += " -k"
    if php: cmd += f" localhost:{port}/test.php"
    else: cmd += f" localhost:{port}/"

    # Run command
    os.system(cmd)

    # Parse output
    with open("out.txt", "r") as f:
        for line in f:
            if field in line:
                f.close()
                os.remove("out.txt")
                return float(line[24:].split()[0])

    os.remove("out.txt")

    raise Exception("Could not find field in output")

def generate_tex(
    xlabel: str,
    ylabel: str,
    xmin: int,
    xmax: int,
    ymin: int,
    ymax: int,
    xtick: list[int],
    ytick: list[int],
    results: dict) -> str:

    output = "\\begin{center}\n\\begin{tikzpicture}\n\\begin{axis}[\n"

    output += "xlabel={{{}}},\n".format(xlabel)
    output += "ylabel={{{}}},\n".format(ylabel)
    output += "xmin={}, ".format(xmin)
    output += "xmax={},\n".format(xmax)
    output += "ymin={}, ".format(ymin)
    output += "ymax={},\n".format(ymax)
    output += "xtick={{{}}},\n".format(",".join(xtick))
    output += "ytick={{{}}},\n".format(",".join(ytick))
    output += "scaled y ticks=false,\n"
    output += "legend pos=north west,\n"
    output += "ymajorgrids=true,\n"
    output += "grid style=dashed,\n"
    output += "]\n"

    for (i, server) in enumerate(results.keys()):
        output += "\\addplot[color={}, mark=square]\n".format(COLOURS[i])
        output += "coordinates {\n"

        for x in range(len(results[server])):
            output += "({}, {}) ".format(x + 1, results[server][x])

        output += "};\n"

    output += "\\legend{{{}}}\n".format(",".join(results.keys()))
    output += "\\end{axis}\n\\end{tikzpicture}\n\\end{center}\n"
    
    return output

if __name__ == "__main__":
    main()
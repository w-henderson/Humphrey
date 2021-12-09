"""
Simple Python script to generate TeX code for embedding the source code.
Also packages up the code to be included in the TeX write-up.
"""

import os
import shutil

EXTENSIONS = [".rs", ".conf"]
OUTPUT_STRING = "\\subsubsection{{{0}}}\n\\inputminted[breaklines]{{rust}}{{pkg/{1}}}\n\n"
SEPARATE_TESTS = True
DIRECTORIES = {
    "humphrey": "Humphrey Core",
    "humphrey-server": "Humphrey Server",
    "humphrey-ws": "Humphrey WebSocket",
    "plugins": "Plugins",
    "examples": "Example Code",
}

shutil.rmtree("pkg", ignore_errors=True)
os.mkdir("pkg")

output = open("pkg/src.tex", "w")
if SEPARATE_TESTS: tests_output = open("pkg/tests.tex", "w")

for (k, v) in DIRECTORIES.items():
    output.write("\\subsection{{{0}}}\n".format(v))
    if SEPARATE_TESTS: tests_output.write("\\subsection{{{0}}}\n".format(v))

    for root, dirs, files in os.walk(f"./{k}"):
        for name in files:
            if any(name.endswith(ext) for ext in EXTENSIONS):
                top_level_dir = root.split(os.sep)[1]
                original_path = os.path.join(root, name)[2:]
                escaped_path = original_path.replace("\\", "/").replace("_", "\\_")
                modified_path = original_path.replace("\\", "-").replace("_", "-")
                
                shutil.copyfile(original_path, "pkg/" + modified_path)

                if "tests" in original_path and SEPARATE_TESTS:
                    tests_output.write(OUTPUT_STRING.format(escaped_path, modified_path))
                else:
                    output.write(OUTPUT_STRING.format(escaped_path, modified_path))

output.close()
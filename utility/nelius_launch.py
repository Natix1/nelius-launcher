import argparse
import json
import os
import platform
import subprocess

parser = argparse.ArgumentParser(
    prog=__name__, description="Launches a minecraft directory via a nelius.meta file"
)

parser.add_argument("-m", "--nelius-meta", dest="nelius_meta")
args = parser.parse_args()

nelius_meta_dir: str = args.nelius_meta
assert len(nelius_meta_dir) > 0, "no root dir specified"


def launch_minecraft(nelius_meta_dir: str):
    with open(nelius_meta_dir, "r") as f:
        meta = json.load(f)

    game_dir = os.path.dirname(os.path.abspath(nelius_meta_dir))
    classpath = []

    for path in meta["classpath_relative"]:
        classpath.append(os.path.abspath(os.path.join(game_dir, path)))

    classpath.append(os.path.abspath(os.path.join(game_dir, "client.jar")))
    separator: str

    if platform.system() == "Windows":
        separator = ";"
    else:
        separator = ":"

    classpath = separator.join(classpath)
    command = [
        "java",
        "-Xmx2G",
        "-cp",
        classpath,
        meta["main_class"],
        "--username",
        "nelius",
        "--version",
        meta["version"],
        "--gameDir",
        game_dir,
        "--assetsDir",
        meta["assets_dir"],
        "--assetIndex",
        meta["asset_index_id"],
        "--uuid",
        "0",
        "--accessToken",
        "0",
    ]

    subprocess.run(command, cwd=game_dir)


if __name__ == "__main__":
    launch_minecraft(nelius_meta_dir)

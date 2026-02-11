import argparse
import json
import math
import os
import platform
import threading
import time
from concurrent.futures import ThreadPoolExecutor
from dataclasses import dataclass
from typing import List, Literal

import requests

MANIFEST_URL = "https://piston-meta.mojang.com/mc/game/version_manifest_v2.json"
RESOURCES_BASE_URL = "https://resources.download.minecraft.net"
CONCURRENT_DOWNLOAD_WORKERS = 64

session = requests.Session()
parser = argparse.ArgumentParser(
    prog=__name__,
    description="Helps download a minecraft version to the given directory",
)

parser.add_argument("-o", "--output", dest="output")
parser.add_argument("-v", "--mc-version", dest="mc_version")

args = parser.parse_args()

output_dir: str = args.output
mc_version: str = args.mc_version

assert len(output_dir) > 0, "no valid output directory specified"
assert len(mc_version) > 0, "no valid minecraft version specified"


@dataclass
class ManifestLatest:
    release: str
    snapshot: str


@dataclass
class ManifestVersion:
    id: str
    type: Literal["snapshot", "release"]
    url: str


@dataclass
class Library:
    library_name: str
    operating_system: Literal["windows", "linux", "osx", "unspecified"]

    download_path: str
    download_url: str


@dataclass
class VersionData:
    id: str
    asset_index_url: str
    asset_index_id: str
    client_jar_url: str
    server_jar_url: str
    libraries: List[Library]
    main_class: str


@dataclass
class Manifest:
    latest: ManifestLatest
    versions: List[ManifestVersion]


def get_manifest() -> Manifest:
    data = requests.get(url=MANIFEST_URL).json()
    versions = []

    for version in data["versions"]:
        versions.append(
            ManifestVersion(id=version["id"], type=version["type"], url=version["url"])
        )

    latest = ManifestLatest(
        release=data["latest"]["release"], snapshot=data["latest"]["snapshot"]
    )

    return Manifest(latest=latest, versions=versions)


def get_version_data(target_version_id: str) -> VersionData:
    manifest = get_manifest()
    raw_manifest_version_data: ManifestVersion

    for version in manifest.versions:
        if version.id == target_version_id:
            raw_manifest_version_data = version
            break
    else:
        raise ValueError("Invalid version provided: not found")

    raw_version_details = requests.get(raw_manifest_version_data.url).json()
    libraries = []

    for raw_library in raw_version_details.get("libraries", []):
        system = "unspecified"

        rules = raw_library.get("rules", [])
        for rule in rules:
            if "os" in rule:
                system = rule["os"].get("name", "unspecified")

        downloads = raw_library.get("downloads", {})
        artifact = downloads.get("artifact")

        if artifact:
            libraries.append(
                Library(
                    library_name=raw_library["name"],
                    operating_system=system,
                    download_path=artifact["path"],
                    download_url=artifact["url"],
                )
            )

    return VersionData(
        id=raw_version_details["id"],
        asset_index_url=raw_version_details["assetIndex"]["url"],
        asset_index_id=raw_version_details["assetIndex"]["id"],
        client_jar_url=raw_version_details["downloads"]["client"]["url"],
        server_jar_url=raw_version_details["downloads"]["server"]["url"],
        main_class=raw_version_details["mainClass"],
        libraries=libraries,
    )


def download_file(url: str, dest_path: str):
    if os.path.exists(dest_path):
        return

    os.makedirs(os.path.dirname(dest_path), exist_ok=True)

    start = time.perf_counter()
    print(f"Downloading {url}...")

    with session.get(url, stream=True) as r:
        r.raise_for_status()
        with open(dest_path, "wb") as f:
            for chunk in r.iter_content(chunk_size=8192):
                f.write(chunk)

    elapsed = math.floor((time.perf_counter() - start) * 100) / 100
    print(f"Finished {os.path.basename(dest_path)} in ~{elapsed}s.")


def download_minecraft_structure(
    version_data: VersionData, executor: ThreadPoolExecutor
):
    os.makedirs(output_dir, exist_ok=True)

    libraries_directory = os.path.join(output_dir, "libraries")
    assets_directory = os.path.join(output_dir, "assets")
    objects_directory = os.path.join(assets_directory, "objects")
    indexes_directory = os.path.join(assets_directory, "indexes")

    os.makedirs(libraries_directory, exist_ok=True)
    os.makedirs(objects_directory, exist_ok=True)
    os.makedirs(indexes_directory, exist_ok=True)

    executor.submit(
        download_file,
        version_data.client_jar_url,
        os.path.join(output_dir, "client.jar"),
    )

    classpath = []
    for library in version_data.libraries:
        if library.operating_system == "windows" and platform.system() != "Windows":
            continue
        if library.operating_system == "linux" and platform.system() != "Linux":
            continue
        if library.operating_system == "osx" and platform.uname().system != "Darwin":
            continue

        download_path = os.path.join(libraries_directory, library.download_path)
        executor.submit(download_file, library.download_url, download_path)
        classpath.append(os.path.relpath(download_path, output_dir))

    asset_index_path = os.path.join(
        indexes_directory, f"{version_data.asset_index_id}.json"
    )
    download_file(version_data.asset_index_url, asset_index_path)

    with open(asset_index_path, "r") as f:
        asset_index_objects = json.loads(f.read())["objects"]

    for object_data in asset_index_objects.values():
        h = object_data["hash"]
        url = f"{RESOURCES_BASE_URL}/{h[0:2]}/{h}"
        full_path = os.path.join(objects_directory, h[0:2], h)
        executor.submit(download_file, url, full_path)

    with open(os.path.join(output_dir, "nelius.meta"), "w") as f:
        json.dump(
            {
                "version": mc_version,
                "asset_index_id": version_data.asset_index_id,
                "classpath_relative": classpath,
                "main_class": version_data.main_class,
                "assets_dir": os.path.relpath(assets_directory, output_dir),
            },
            f,
        )


if __name__ == "__main__":
    version_data = get_version_data(mc_version)
    print(f"Starting download for Minecraft {mc_version}...")
    with ThreadPoolExecutor(max_workers=CONCURRENT_DOWNLOAD_WORKERS) as executor:
        download_minecraft_structure(version_data, executor)

    print("Successfully downloaded all game files")

import requests
import shutil
import sys
import hashlib

from pathlib import Path

def check_and_download_web(file_to_download: str, output_folder: Path):
    temp_name = str(hashlib.md5(file_to_download.encode('utf-8')).hexdigest())
    print(f'file to download: {file_to_download}')
    print(f'hexdigest hash {temp_name}')

    output_folder.mkdir(parents=True, exist_ok=True)
    target_path = output_folder / Path(temp_name)

    r = requests.get(file_to_download, stream=True)

    if r.status_code == 200:
        with open(target_path, 'wb') as f:
            r.raw.decode_content = True
            shutil.copyfileobj(r.raw, f)

    print(f'downloaded: {temp_name}')



import timeit
from pytube import Search
import json
import argparse

parser = argparse.ArgumentParser()
parser.add_argument("--query", type=str)
args = parser.parse_args()
s = Search(args.query)
youtube_data = [{"title": v.title, "video_id": v.video_id} for v in s.results]
print(json.dumps(youtube_data))

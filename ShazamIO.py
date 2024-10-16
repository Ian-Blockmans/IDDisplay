from shazamio import Shazam
import json
import asyncio
import sys

async def shazam_rec():
    shazam = Shazam()
    out = await shazam.recognize(sys.argv[1])  # rust version, use this!
    print(json.dumps(out, indent=2))

asyncio.run(shazam_rec())
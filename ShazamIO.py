from shazamio import Shazam
import json
import asyncio
import sys

async def shazam_rec():
    shazam = Shazam()
    out = await shazam.recognize(sys.argv[1])  # rust version, use this!
    #out = await shazam.recognize("MPH-CrowdRolling.ogg")  # rust version, use this!
    #print(json.dumps(out, indent=2), file=shazamJson)
    print(json.dumps(out, indent=2))
    #return json.dumps(out, indent=2)

asyncio.run(shazam_rec())
#loop = asyncio.get_event_loop()
#loop.run_until_complete(shazam_rec())
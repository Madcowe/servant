# Issues and Updates

## Issues

### Cache

- The below issus have not been resolved.
- Clearing the cache does not seem to work, if I download a file succesfully and then clear the cache, if I got back to the same url it does not seem to download the file again.
- If a download fails for soem reason, the url seems to get cached to just show the same error rather then trying to download the file again.

### Download failures

- The below are still failing...they work fine in ant-cli and I even managed to download the failed chunk in the app (see chunk downloads) so there must be something of with this approach can we compare to how it is down in ant-cli.
- I have a couple of files that I can't seem to download, they retrieve the datamap but and errors says a chunk is missing. It works fine in Ant- CLI. These are both mp3 files ... the files the worked are jpeg but I don't see why that would make a difference. Here are the two urls
ant://00ac7cbe1fe3e49fcd9e490eb313fabc2fe4407e67196292e961c3b34e9b1afa which says it is missing chunk e4d0508a9f0cf102a21871a931cb08be87375245a699c74f23fea00c3a0861ae
ant://18a1f9923d61dcd03266b06bead66c39fc75aea83c42be74d95a32cb42cf89e9 which says it is missing chunk 50d9d85885067438af8120dab20f76f8fdea1d621f65c70439c2b8129928a60d

both those chunks download fine with ant cli

### Chunk download
I get a Unkown cotent type (application/octet-stream) error message after downloading a chunk for a direct chunk address...from the teminal output it looks like it downloaded

### URL format

- The below is not yet working
- It would be good if the url could take a slash and filename after the address, and then the name would be used for the file. For example (this is a real address)
ant://3387bb85853e2583b92da167cce3fd46dff574ff7fe8f85d02c1f65d54535c44/elite_log.jpeg.


## Updates

### UI progess indicators
- The progress is only shown in the terminal we need it to be within the app...is there anything like a status bar we could use within servo. Or could we make an updating html pages that shows the progress?



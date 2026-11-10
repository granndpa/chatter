# Server asset pack maintenance

The November 2026 full pack names its twelve header tiles
`header-r1-c1.png` through `header-r2-c6.png`. Discover these by row and
column, excluding `header-complete.png`, and map them to tiles 1 through
12 in row order. A numeric-filename filter skips the entire header.

Maintenance completed on 10 November 2026:

- Uploaded and verified all twelve missing tiles in each brand's server.
- Refreshed all 22 static and animated pack emoji in each server from the
  supplied full-pack ZIP. Created each replacement before removing the old
  pack emoji, and preserved unrelated expressions.
- Updated the private emoji mapping after each successful upload.
- Repaired the existing latest Chatter release post and read it back.
- Repeated the repair to verify that it creates no duplicate expressions
  or release posts.
- Rejected GIF input in the private sticker upload helper. Sticker
  uploads must use PNG, APNG or Lottie JSON.

Both servers have five stickers and no remaining sticker slots. Seven
additional PNG stickers from each brand's pack remain unuploaded. The five occupied sticker slots in each server now use the new PNG artwork;
the selection of stickers was preserved.

Credentials, server and channel identifiers, emoji mappings, release
content, and maintenance helpers remain in the ignored private directory.

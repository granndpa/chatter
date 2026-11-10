# Composer stickers

Open Stickers beside the GIF button, search by sticker name, tags or server,
and choose a sticker. It is staged beside the draft; remove it with the
close button, or press Send to include it with the text, reply and files.
An empty text message can send a sticker by itself.

The picker lists stickers from the linked account's servers, fetching at
most four servers concurrently. Unavailable stickers cannot be chosen.
Discord checks permissions and Nitro requirements when a sticker is sent.
The virtualized list requests preview images only as rows are instantiated.

The normal outgoing queue carries the selected ID as `sticker_ids` in
`POST /channels/{channel}/messages`. Failed sends preserve the selection
for retry. A successful send clears only the sticker actually sent, so a
new selection made while a send is pending survives.

The picker currently targets the main composer. Open a thread as its own
channel to choose a sticker for it. Stickers are not attached to edits or
to helper messages sent independently of the composer draft.

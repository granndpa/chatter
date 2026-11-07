# Connected Apps

Settings lists Connected Apps under Games & Apps, with a Connections entry.
The add buttons offer Spotify, GitHub, Reddit, Steam, YouTube, Twitch,
Epic Games and Riot Games. Other existing connections remain manageable.
Provider marks are bundled from [Simple Icons](https://github.com/simple-icons/simple-icons)
(CC0); the domain mark is a globe. The same marks appear in profiles, where
connected domains open their HTTPS website, like connected service accounts.
It fetches `GET /users/@me/connections` when opened and on Refresh. Each card
shows the account and provider, Display on profile, and, for Spotify, Display
Spotify as your status. Changes wait for Discord's answer and read the list
back. Removing an account asks for confirmation first. Unknown providers still
get cards and can be hidden or removed.

The page uses these requests:

- `PATCH /users/@me/connections/{type}/{id}` with just `visibility: boolean`
  or `show_activity: boolean`.
- `DELETE /users/@me/connections/{type}/{id}` with no body.
- `GET /connections/{type}/authorize`, taking `url` from the response and
  keeping its state in memory before opening the browser.
- `POST /connections/{type}/callback` with `code`, `state`, `insecure: false`,
  `friend_sync` (true only for Facebook among the offered providers), plus
  `openid_params` and `iss` when supplied by the provider.

Verified on 7 November 2026 against Discord's public
[web bundle](https://discord.com/assets/web.72656029554c0370.js), modules
77468 (authorize, changes and removal), 306677 (callback), and the connection
[callback page](https://discord.com/assets/a299b92e0da9d001.js), module 291007.
The latter collects `openid.*` parameters, uses `oauth_verifier` before `code`,
and forwards `iss`. The parser also accepts state inside `openid.return_to`
for Steam. Live authorize URLs could not be inspected without authentication.

## Adding a connection

Finish login in the browser, then use Refresh. A browser already signed into
the correct Discord account can finish itself. If it cannot, paste the complete
`https://discord.com/connections/{type}?…` redirect URL into Static and choose
Finish connection. Discord's error page preserves the query, so its
`/connections/{type}/error?…` URL is also accepted. The URL is masked, never
saved or logged, and cleared from the input on submission. It must match the
provider and state of the pending login, which expires after ten minutes.
Duplicate parameters, other hosts, fragments and provider errors are rejected.

Live provider logins have not been exercised by the implementation's tests.
Some providers may require additional two-way linking or a continuation.
Static does not try to invent those steps. A browser may consume the code or
replace its URL with a success page before it can be pasted; Refresh first,
then start another login if necessary. Bluesky's handle prompt, domain
verification, and Authorized Apps are outside this page.

Spotify changes wake its presence worker, clear the cached Spotify token and
current activity, and read the connection setting again. The profile switch
and status switch are independent. Changes made in another Discord client
are read on Refresh; no background polling of the Connections list is added.
The account popover also shows the account's actual username and Spotify's
song, artist, album, artwork and progress. Activity comes from the local
Spotify bridge or the account's gateway presence; it is not saved as a stale
listening activity in the disk cache.

## Switching accounts

The account popover and the sign-in screen offer Switch Accounts. Add account
restarts into a separate login namespace and opens the existing sign-in flow.
Selecting a saved account restarts into its namespace without logging out or
revoking the previous account. Switching ends the current call.

`discord-accounts.json` holds public names and ids plus the selected namespace.
Tokens and cache keys remain in separate OS credential-store entries. The
default namespace keeps the original entries, so an existing login needs no
migration. Other namespaces use separate cache directories. Selection changes
take effect only in the next process, after this process's workers stop.

## Checks

`cargo test -p static-discord` covers settings and redirect validation.
`cargo test -p static-core connections` covers pending requests, confirmation
and stale responses. `manages_connected_accounts` in `tests/fake_server.rs`
checks the service's request bodies against `testing/fake_discord.ts`; run it
with Bun on PATH and `--ignored`. The fake records requests at
`GET /_control/connections` and emits `USER_CONNECTIONS_UPDATE` on changes.

# Bot forms

Buttons, dropdown selections and slash commands can open a bot form.
Discord delivers response type 9 to the user session as
`INTERACTION_MODAL_CREATE`, rather than returning the form from the HTTP
interaction request. Chatter correlates the gateway nonce with a recent
interaction and verifies its application and channel before opening it.

The dialog supports short and paragraph text inputs, including defaults,
placeholders, optional fields and length constraints, in action rows or
label wrappers. Unsupported component types show an explicit notice.
Answers remain available after an HTTP failure. Submit is disabled while
the request is pending; cancelling closes the form without sending it.

Submission uses interaction type 5 with the original interaction ID,
modal custom ID, source application/channel/server, and component IDs and
answers. Presentation-only properties are omitted. The service adds the
current gateway session ID, as for buttons and slash commands.

Protocol references:

- https://docs.discord.food/gateway/gateway-events#interaction-modal-create
- https://docs.discord.food/interactions/receiving-and-responding

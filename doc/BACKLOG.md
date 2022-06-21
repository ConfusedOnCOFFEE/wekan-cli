# Backlog

- Currently there is not a good way on when a name argument is expected or not. I want to change that and make it more transparent and coherent in all subcommand but this takes time.
- Return messages schema.
- Exit and error codes are random.
- Apply all traits (CommonRunner, SubcommandRunner...) to the Runners.
- Remove vector in Display.
- Remove format field in runners.
- Better LOG LEVEL design.


# Backlog (Missing features against API)

- Labels for card update doesn't work. Somehow the parsing panics or my derive is bad.
- If an artifact is created, the age of the store should be removed or atleast make it in a way, that the new artifact can be found.
- emacs integration.
- Remove store feature and always disable it.
- Find a way to E2E without extra build set.
- Configure different build setups.


This is basiclly a list of the API endpoints, which were not touched. I don't know, if I do them or not.


Boards:

- get_public_boards
- delete_board
- add_board_label
- get_board_attachments
- set_board_member_permission
- get_boards_count
- update board information (PUT /board/{id})

Cards:

- edit card with more options
- get_swimmlane_cards

Checklists:

- get_all_checklists
- new_checklist
- get_checklist
- delete_checklist

- ChecklistItems
- Swimmlanes
- Card Comments
- Custom Fields
- Integrations
- Schemas
- Users


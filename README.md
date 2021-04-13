# qbot
Simple client-side chatbot for Twitch used for queueing chat members for community engagement. 

[![Actions Status](https://github.com/trollham/qbot/workflows/CI/badge.svg)](https://github.com/trollham/qbot/actions)

## Building
The application is built in two steps:
```bash
cargo build
cd www && yarn build
```
## Running
Create a file called `Settings.toml` in the same location as the qbot executable that contains the following:
```toml
name = "<bot_name>"
channel = "<channel_name>"
```
Replace `<bot_name>` with the account name the bot is using in chat, and `<channel_name>` with the channel the bot is joining. 

You must click `Connect to chat` after starting qbot. This will take you to the Twitch Authentication page, which requires you to authenticate as the same account as the `name` field in the `Settings.toml` file. This allows the bot to sign into Twitch and join the specified channel. 

qbot starts with the queue closed. Click the Open button to allow users in chat to begin joining the queue. 

## Chat Commands
* `!join` - User: add themselves to the queue. A user is not allowed to be in the queue multiple times. They must be removed by either `!leave`-ing themselves, or via the frontend, before they can join again. `!join`ing more than once will result in the bot responding with that user's position as if the user did used the  `!place` command.
* `!leave` - User: remove themselves from the queue.
* `!next` - User: peek at the upcoming group. Does not modify the queue.
* `!place` - User: get their position in the queue, with an estimated wait time.

## Roadmap
- [x] Add ability to change party size. Not really necessary in its current use case, this would just make the bot less brittle
- [ ] Separate backend onto remote host.
  - [ ] Create frontend application
  - [ ] Switch to stronger oauth flow
  - [ ] Create website so the queue owner, moderators, and community members can monitor/interact with the queue
- [ ] Allow for reordering queue members
- [ ] add moderator controls to the chat bot

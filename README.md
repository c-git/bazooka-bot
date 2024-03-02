# Bazooka Bot

This is the source code for the bot for the [Bazooka Alliance Discord Server](http://discord.gg/uQVy7BH)

This is the rust replacement for the bot [originally](https://github.com/fone-git/bazooka-bot) written in python.

# List of TODOs for pending features

# Unranked unranked

- [x] `/unranked idea`
  - [x] `add(description)`
  - [x] `edit(id, new_description)`
  - [x] `remove(id)`
  - [x] `vote(id)`
  - [x] `unvote(id)`
  - [x] `vote_all`
  - [x] `unvote_all`
- [ ] `/unranked score`
  - [x] `set(score)`
  - [x] `remove`
  - [x] `results`
  - [ ] `other_set(user, score)` (Auth Req)
  - [ ] `other_remove(user)` (Auth Req)
  - [ ] `set_message(message)` (Auth Req)
- [x] `/unranked schedule_reset(date_time)` (Auth Req)

## Scheduled Actions

At the start of unranked each season the following should happen.
It will be scheduled manually each season by an officer.

- [ ] Verify user is authorized
- [ ] Print current info before clearing
- [ ] Clear the scores for the last season
- [ ] Determine the winning idea (Highest votes or lower ID if tied)
- [ ] Set the new unranked message based on the winning idea (Should show with the scores so people know what the unranked is).
- [ ] Clear the ideas. Keep any ideas with more than `idea_keep_threshold` votes except for the winning one.
- [ ] Announce the new unranked challenge

# Nice to have

- [ ] Add web page (with info and to wake up bot) https://docs.shuttle.rs/templates/tutorials/custom-service
- [ ] Add event handler to see when ppl leave https://github.com/serenity-rs/poise/blob/current/examples/event_handler/main.rs
- [ ] Restrict unranked to that channel
- [ ] Send a status messages when it connects (including the version)
- [x] Change results (vote counts and leader board) to (embeds)[https://docs.rs/poise/latest/poise/serenity_prelude/struct.CreateMessage.html#examples]
- [ ] Add ability for owner to download the data files to facilitate testing before uploading a version

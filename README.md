# Bazooka Bot

This is the source code for the bot for the [Bazooka Alliance Discord Server](http://discord.gg/uQVy7BH)

This is the rust replacement for the bot [originally](https://github.com/fone-git/bazooka-bot) written in python.

# Features / TODO list

<!-- Leave completed items as a feature list / what is being considered for implementation -->

Note: Some checked off items my not be complete only started but at the time of writing (2024-03-02) there is only one item like that, "schedule_start_unranked"

# Unranked event

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
  - [x] `set_message(message)` (Auth Req)
- [x] `start_event` (See [event start](#event-start) for details) (Auth Req)
- [x] `/unranked schedule_start_event(date_time)` (Auth Req)

## Event Start

At the start of unranked each season the following should happen.
It should be scheduled manually each season by an officer.

- [x] Verify user is authorized
- [x] Print current info before clearing
- [x] Clear the scores for the last season
- [x] Determine the winning idea (Highest votes or lower ID if tied)
- [x] Set the new unranked message based on the winning idea (Should show with the scores so people know what the unranked is).
- [x] Clear the ideas. Discard any ideas less than or equal to `discard_threshold` votes plus the winning one.
- [x] Announce the new unranked challenge

# Nice to have

- [ ] Add ability for owner to download the data files to facilitate testing before uploading a version
- [ ] Add message ID to the trace at ingress
- [ ] Add event handler to see when ppl leave https://github.com/serenity-rs/poise/blob/current/examples/event_handler/main.rs
- [x] Sanitize input for markdown like `**` for example rn causes problems with bolding the ideas
- [ ] Add web page (with info and to wake up bot) https://docs.shuttle.rs/templates/tutorials/custom-service
- [ ] Restrict unranked to that channel
- [x] Send a status messages when it connects (including the version)
- [x] Change results (vote counts and leader board) to (embeds)[https://docs.rs/poise/latest/poise/serenity_prelude/struct.CreateMessage.html#examples]
- [ ] Make reset a 2 stage process with a confirmation
- [ ] Add a permission that can be used as a default_permission to tell slash commands just not to show if a user doesn't have it instead of returning a no permissions message

# Bazooka Bot

This is the source code for the bot for the [Bazooka Alliance Discord Server](http://discord.gg/uQVy7BH)

This is the rust replacement for the bot [originally](https://github.com/fone-git/bazooka-bot) written in python.

# List of TODOs for pending features

# Unranked unranked

- [ ] `/unranked idea`
  - [ ] `add(description)`
  - [ ] `edit(id, new_description)`
  - [ ] `remove(id)`
  - [ ] `vote(id)`
  - [ ] `unvote(id)`
  - [ ] `vote_all`
  - [ ] `unvote_all`
- [ ] `/unranked score`
  - [ ] `add(score)`
  - [ ] `edit(new_score)`
  - [ ] `remove`
  - [ ] `other_add(user, score)` (Auth Req)
  - [ ] `other_edit(user, new_score)` (Auth Req)
  - [ ] `other_remove(user)` (Auth Req)
- [ ] `/unranked results`
- [ ] `/unranked schedule_reset(date_time)` (Auth Req)

## Scheduled Actions

At the start of unranked each season the following should happen.
It will be scheduled manually each season by an officer.

- [ ] Verify user is authorized
- [ ] Clear the scores for the last season
- [ ] Determine the winning idea (Highest votes or lower ID if tied)
- [ ] Set the new unranked message based on the winning idea (Should show with the scores so people know what the unranked is).
- [ ] Clear the ideas. Keep any ideas with more than `idea_keep_threshold` votes except for the winning one.

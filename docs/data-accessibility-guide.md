# Data Accessibility Guide

Updated: Oct 24, 2026

This document details when and how data that you explicitly enter is
accessible to others.

## Account and contact information

Player email addresses and phone numbers are not displayed on public
profiles. Passwords and OAuth credentials are not published.

A team's **public contact email** is visible to anyone on its profile.
The team owner can add, change or remove this address in Edit profile.
Creating a new team with Google does not automatically publish the Google
email address. Existing team contact addresses remain visible until the
owner changes or removes them.

## Injuries
 
New injuries are private by default. Injuries set to **Public** are visible
to everyone on the player's profile. **Private** injuries are hidden from
the public profile; the player can still view and edit them.

Active injuries, including private ones, are shown to authorized match
referees during match setup at events the player is registered for. Marking
an injury healed removes it from this active-injury information. Changing
the default for new injuries does not change existing records.

In either case, injury (description, date, and status) may be
published anonymously. This could be in the form of statistics or
even a full release of information, but no identifying information
nor structure will ever be published (ie, your username will not be
attached, we will never group injuries by user, etc.).

## Penalties

 - are visible to target forever on their profile
 - are visible forever on target's profile to all explicitly listed head
   refs and TOs for the tournament at which they were logged
 - are visible during tournament on target's profile to all head refs for
   the tournament at which they were logged
 - are visible to head refs when starting, running, or viewing a match in 
   the same tournament in which they were logged
 - may be shown in aggregate statistics, but timestamp will be rounded
   to the day, and no information about author or target will not be
   shown besides the target type (team or player)
   
 The [user docs](docs.md#penalties) have more information on the types
 of notes, where they can be seen, and how they get written.

!!! warning "Future Changes"  
    the promises about penalty privacy may change in the future, but
    a) users will be notified of any such changes, and b) they will
    not apply retrospectively (ie, these rules will always apply to
    notes written under these rules)

## Encryption/sysadmin disclaimer

All of this being said, none of the data dealt with by Arctos is
actually encrypted for server side operations - as described in the
second paragraph of the privacy policy, anyone with server access
could view private information with some minimal technical knowledge
(though authentication data is actually secure). The server itself is
very secure, but you do have to trust the sysadmins to not leak
information (to be clear, they will not).

# ArchiveMail (in Rust)

A stripped down "rewrite" of [archivemail](https://archivemail.sourceforge.net)
in Rust. The old archivemail utility is written for Python2 and uses an older
Python mail library, as such most distros don't package it anymore.

Currently, it only supports moving mail to a different maildir location or
deleting emails older than X days (default is 31).

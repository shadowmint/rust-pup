# TODO

## Core

- Add a `-c` flag to read a list of environment variable overrides.
- If a task is missing, print the parent, not the missing key for debugging.
- Add -vv for extra verbose on the dryrun to print env, we don't really care normally.
- Allow root folder as a fallback if there are no versions

## Workers

- Add an 'auto' worker.
- Change from 'main.yml' to 'worker.yml' for all workers.

### pup-worker-auto

This worker should be able to be given a folder, discover all the subfolders in it, and execute
them as tasks, without explicitly requiring stuff.

eg. Given a sub-folder of cargo tasks, go in there and execute `pup` on each of them, with some
given task.

Or should this be a core task?

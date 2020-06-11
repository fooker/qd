# qd - Local filesystem job queue
`qd` (pronounced *queue-dee*) is a job (or task) manager only utilizing the local filesystem.
Each job managed is represented by a filesystem directory containing all the assets defining the unit of work.
For each enqueued task, a specified command will be executed while using the jobs directory as the current working path.

## Usage
`qd` can be called with one of the following operations:
* `qd daemon` starts the daemon executing the jobs as they are enqueued.
It will watch the queue for new jobs and execute them one after another as the come in.
If a job failed, it will be re-enqueued after a certain amount of time.
* `qd push` creates a new job directory and executes a given command inside this directory to create the jobs required assets.
The command will block until the job is created and only if creation is successful, the job will be pushed to the queue.
* `qd stats` prints out some basic stats about the queue.

## Building
`qd` requires [rust](https://www.rust-lang.org/)  and [cargo](https://doc.rust-lang.org/cargo/) to be installed.
To build the project from source just run the following command:
```
cargo build --release
```
The final build result will be found in `target/release/qd`.
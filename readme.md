Run Kill Run
============

*Run* an executable Program. When the exe file changes *kill* the old process. *Run* the new exe again.

## Usage

- Invoke as:
    ```bash
    rkr EXE [ARGS *]
    ```
-   EXE must be a (relative/absolute) path to an executable file. (PRs that look through PATH welcome).
-   rkr will spawn EXE with ARGS. Whenever EXE changes, the target is killed and started again
-   rkr exits with:
    -   42 on error
    -   43 if the target exited due to a signal
    -   return code of the target process if target exits normally.

## Use

I use it while developing a client/server application to run the server. Most problems are in the client, so I start
that from my IDE.

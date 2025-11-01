- general structure of command line
    ```
    wxrust [...global options] <cmd> [...command options]
    ```

- show help:
    - general help about tool (no command details)
        ```
        wxrust -h
        wxrust help
        ```

    - command details
        ```
        wxrust <cmd> -h
        wxrust <cmd> help
        ```

- global options
    - change credentials file location
        ```
        wxrust -c credentials_path <cmd> ...
        wxrust --credentials credentials_path <cmd> ...
        ```

    - force login, ignore cached auth token
        ```
        wxrust -a --force-authentication <cmd> ...
        ```

- listing workouts

    - general format of command
        ```
        wxrust list [-d|--details] [-s|--summary] [-a|--all|<dates>...]
        ```

    - list all dates of workouts
        ```
        wxrust list
        wxrust list -a
        wxrust list --all
        ```

    - restrict range to some year
        ```
        wxrust list 2025
        ```

    - restrict range to year range (all workouts in range of years inclusive)
        ```
        wxrust list 2024-2025
        ```

    - restrict range to multiple some months (arbitrary number of months can be listed)
        ```
        wxrust list 2025.10 2025.11
        wxrust list 2025/10 2025/11
        wxrust list 202510 202511
        ```

    - restrict range to month range (all workouts in range of months inclusive)
        ```
        wxrust list 2025.10-2025.11
        wxrust list 2025/10-2025/11
        wxrust list 202510-202511
        ```

    - by default show only the dates of workouts (no details or summary), using YYYY-MM-DD

    - show details of each workout listed (full workout details)
        ```
        wxrust list -d ...
        wxrust list --details ...
        ```

    - show summary of each workout (one line summary of workout)
        ```
        wxrust list -s ...
        wxrust list --summary ...
        ```


- showing workout details

    - general format of command
        ```
        wxrust show [-s|--summary] <date>
        ```

    - showing one day

    ```
    wxrust show <date>
    ```

    - showing only summary

    ```
    wxrust show --summary <date>
    ```



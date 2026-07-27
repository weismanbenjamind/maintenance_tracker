# Maintenance Tracker

`maintenance_tracker` is a lightweight CLI tool to track auto maintenance.

## Installation

Simply clone the repo, checkout your desired version tag, build the binary, put it in desired deployment location (maybe on your `PATH`), and delete the repo. _Note: the installation assumes you have git installed and can clone a repo and also have the rust toolchain (e.g. cargo) installed._

For example:

```sh
git clone https://github.com/weismanbenjamind/maintenance_tracker.git
git checkout v1.0.0
cargo build -r
mv ./target/release/maintenance_tracker /your/desired/deployment/location/maybe/on/PATH
rm -r maintenance_tracker
```

## Usage

The below steps outline how to use the tool.

### Step 1: Create a Maintenance Log

By default `maintenance_tracker` will look for a `maintenance_log.toml` in your present working directory. To automatically scaffold the log run `maintenance_tracker log`. This command will automatically spin you up with valid maintenance log at a path of your choosing (defaulting to a `maintenance_log.toml`):

```toml
[services.example_service]
name = "Example Service"
notes = [
    "This is an example service",
    "Fill out this log with your own services in a similar fashion to this example",
    "Can use 'maintenance_log init' to initialize services with proper scaffolding",
    "Notes are optional",
    "Previous services are optional",
    "Everything else must be filled out (e.g. name, next service, service interval, and an id defined by [services.id] in the .toml file)",
    "Once your services have been initialized delete this example",
]

[services.example_service.service_interval]
miles = 5000
months = 6

[services.example_service.next_service]
miles = 78000
date = "2026-10-23"

[[services.example_service.previous_services]]
miles = 73000
date = "2026-04-24"

[[services.example_service.previous_services]]
miles = 70000
date = "2025-10-24"
```

#### Notes on the log file

The following fields must be present:
- `name`
- `service_interval`
- `next_service`
- `id`
  - The id is denoted by the `toml` tables
  - For example, in the above log contains a single service `example_service` denoted by `[services.example_service]`

The following fields are optional:
- `previous_services`
- `notes`

### Step 2: Filling out the Maintenance Log

Running `maintenance_tracker init` will automatically read your log file and initialize a service to be tracked. To initialize an oil change service using 5000 mile/6 month intervals, a next service at 80000 miles/on 2026-11-24, a couple of notes (these are optional), and a couple of previous services (also optional) run the following:

```sh
maintenance_tracker init "Oil Change" oil_change 5000 6 80000 2026-11-24 -n "Doing 5000 mile intervals" --notes "Doing 6 month intervals" --previous-services "70000;2025-07-24" -p "75000;2026-1-24"
```

Now looking at your log file the following service will be present:

```toml
[services.oil_change]
name = "Oil Change"
notes = [
    "Doing 5000 mile intervals",
    "Doing 6 month intervals",
]

[services.oil_change.service_interval]
miles = 5000
months = 6

[services.oil_change.next_service]
miles = 80000
date = "2026-11-24"

[[services.oil_change.previous_services]]
miles = 75000
date = "2026-01-24"

[[services.oil_change.previous_services]]
miles = 70000
date = "2025-07-24"
```

***This same process can be followed to initialize all services that are desired to be tracked.***

Let's also track the transmission fluid service with the following (No notes or previous services):

```sh
maintenance_tracker init "Transmission Fluid" transmission_fluid 30000 36 90000 2027-11-24
```

Now the following should also be present in the maintenance log:

```toml
[services.transmission_fluid]
name = "Transmission Fluid"

[services.transmission_fluid.service_interval]
miles = 30000
months = 36

[services.transmission_fluid.next_service]
miles = 90000
date = "2027-11-24"
```

#### Deleting a service

To delete a service from the log simply run `maintenance_tracker delete <ID>`. Now that we have actual services in our log (`oil_change` and `transmission_fluid`) - let's go ahead and delete the example service with:

```sh
maintenance_tracker delete example_service
```

Then to confirm that only the oil change is present run `maintenance_tracker list` and  `oil_change` and `transmission_fluid` should be present in the output (and the output `example_service` should be missing).

### Step 3: Inspecting Services

The following commands can be used to inspect services and service intervals.

#### Status

Status is the most useful command. It checks next service thresholds compared to the current miles on the vehicle and a given date. An `id` can be supplied to get the status for a single `maintenance_item`. If the `id` is omitted, the status for all maintenance items will be shown. A `--date` can also be supplied to override the date each maintenance item's next service date is compared to. `--date` defaults to today's date.

Example using the log file we constructed in step 2 above:

```sh
# Status of all services vs today's date assuming 70000 miles on the vehicle and that today's date is 2026-07-24
maintenance_tracker status 70000

# Outputs:
# Name: Oil Change
# Next service (miles): 80000 Miles
# Current mileage: 70000 Miles
# Miles until next service (Next Service Miles - Current Miles): 10000 Miles
# Next service date: 2026-11-24
# Today: 2026-07-24
# Days until next service (Next Service Date - Today): 123 Days

# Name: Transmission Fluid
# Next service (miles): 90000 Miles
# Current mileage: 70000 Miles
# Miles until next service (Next Service Miles - Current Miles): 20000 Miles
# Next service date: 2027-11-24
# Today: 2026-07-24
# Days until next service (Next Service Date - Today): 488 Days
```

```sh
# Status of only an oil change assuming 70000 miles on the vehicle and comparing to 2026-07-24
maintenance_tracker status 70000 --id oil_change --today 2026-07-24

# Outputs:
# Name: Oil Change
# Next service (miles): 80000 Miles
# Current mileage: 70000 Miles
# Miles until next service (Next Service Miles - Current Miles): 10000 Miles
# Next service date: 2026-11-24
# Today: 2026-07-24
# Days until next service (Next Service Date - Today): 123 Days
```

#### Detail

`Detail` prints details about a service. For example using the log file we constructed in step 2 above:

```sh
# Command:
maintenance_tracker detail oil_change

# Outputs:
# Id: oil_change
# Name: Oil Change
# Service Interval Miles: 5000
# Service Interval Months: 6
# Next Service Miles: 80000
# Next Service Date: 2026-11-24
# Previous Services:
#   - 2025-07-24/70000 miles
#   - 2026-01-24/75000 miles
# Notes:
#   - Doing 5000 mile intervals
#   - Doing 6 month intervals
```

#### Next

`Next` will get the next service miles and date for a given maintenance item. For example using the log file we constructed in step 2 above:

```sh
# Command:
maintenance_tracker next transmission_fluid

# Outputs:
# Next Service Miles: 90000
# Next Service Date: 2027-11-24
```

#### Diff

A `diff` can be performed vs a threshold miles and/or date (e.g. what services need done by this mileage and/or date). A diff can also be performed vs a miles and/or months (e.g. what services need done in the next X number of miles and/or Y number of months). An `id` can be provided to diff a single maintenance item. If no `id` is provided the diff will be performed on all maintenance items.

Example using the log file we constructed in step 2 above:

```sh
# Threshold diff
# See if an oil change is due by 86000 miles and/or 2027-02-22
# Assuming 84000 miles on the vehicle
maintenance_tracker diff oil_change threshold -m 86000 -c 84000 -d 2027-02-22

# Outputs:
# Name: Oil Change
# Next service miles: 85000 Miles
# Current miles: 84000 Miles
# Miles until next service (Next Service - Current): 1000 Miles
# Next service date: 2027-01-22
# Date threshold: 2027-02-22
# Days until next service (Next Service Date - Date Threshold): -31 Days
```

```sh
# Interval diff
# See if transmission fluid service is due in the 7000 miles or 13 months
# Assuming 84000 miles on the vehicle and that today's date is 2026-11-25
maintenance_tracker diff transmission_fluid interval -m 7000 -c 84000 --months 13 -t 2026-11-25

# Outputs:
# Name: Transmission Fluid
# Next service miles: 90000 Miles
# Current miles: 84000 Miles
# Miles until next service (Next Service - Current): 6000 Miles
# Next service date: 2027-11-24
# Date threshold: 2027-12-25
# Days until next service (Next Service Date - Date Threshold): -31 Days
```

```sh
# See which services are due by 86000 miles and/or 2027-02-22
# Assuming 84000 miles on the vehicle
maintenance_tracker diff threshold -m 86000 -c 84000 -d 2027-02-22

# Outputs:
# Name: Oil Change
# Next service miles: 85000 Miles
# Current miles: 84000 Miles
# Miles until next service (Next Service - Current): 1000 Miles
# Next service date: 2027-01-22
# Date threshold: 2027-02-22
# Days until next service (Next Service Date - Date Threshold): -31 Days
```

```sh
# See which services are due in the 7000 miles or 13 months
# Assuming 84000 miles on the vehicle and that today's date is 2026-11-25
maintenance_tracker diff interval -m 7000 -c 84000 --months 13 -t 2026-11-25

# Outputs:
# Name: Oil Change
# Next service miles: 85000 Miles
# Current miles: 84000 Miles
# Miles until next service (Next Service - Current): 1000 Miles
# Next service date: 2027-01-22
# Date threshold: 2027-12-25
# Days until next service (Next Service Date - Date Threshold): -337 Days

# Name: Transmission Fluid
# Next service miles: 90000 Miles
# Current miles: 84000 Miles
# Miles until next service (Next Service - Current): 6000 Miles
# Next service date: 2027-11-24
# Date threshold: 2027-12-25
# Days until next service (Next Service Date - Date Threshold): -31 Days
```

### Step 4: Completing a Service

Completing a service is simple. Just provide the ID of the service to be completed, the miles it was completed at, and an optional date parameter for the date it was completed on. Note, the date parameter is optional and defaults to today's date.

Example using the log file we constructed in step 2 above:

```sh
# Complete an oil change at 80000 miles on 2026-07-24
maintenance_tracker complete oil_change 80000 -d 2026-07-24

# Outputs:
# Marked Oil Change as complete at 80000 miles on 2026-07-24
# Updated next service to 85000 miles or on 2027-01-22
```

Note that `complete` will automatically use the service interval to update the next service. Also `complete` will always round down the date when doing date arithmatic. There is some error in the date arithmatic when converting between months, days, weeks, etc; intervals between them; and finally into dates . Rounding down was a design choice made to ensure the error always results in a situation where the service should be performed a little early vs. a little late


### Step 5: Updating Services

To update a service use the `update` command; `update` allows for the following modifications:

- `name`
- `id`
- `service-interval`
  - `miles`
  - `months`
- `next-service`
  - `miles`
  - `date`
- `notes`
  - `append`
  - `replace`
  - `insert`
  - `remove`
  - `clear`
- `previous-services`
  - `append`
  - `replace`
  - `remove`
  - `clear`

Example using the log file we constructed in step 2 above:

```sh
# Change the service interval on the oil change service
maintenance_tracker update oil_change service-interval -m 4000 --months 5

# Add a couple notes to the oil change about the service interval update
# First clear the current notes
maintenance_tracker update oil_change notes clear
maintenance_tracker update oil_change notes append "Doing 4000 mile intervals" "Doing 5 month intervals"

# Update the 70000 mile service on the oil_change to be 69500
maintenance_tracker update oil_change previous-services replace -m 70000 -n 69500
```

After the above changes have been made, inspect the oil change service with:

```sh
# Detail the oil change to see updates that were just made
maintenance_tracker detail oil_change

# Outputs:
# Id: oil_change
# Name: Oil Change
# Service Interval Miles: 4000
# Service Interval Months: 5
# Next Service Miles: 85000
# Next Service Date: 2027-01-22
# Previous Services:
#   - 2025-07-24/69500 miles
#   - 2026-01-24/75000 miles
#   - 2026-07-24/80000 miles
# Notes:
#   - Doing 4000 mile intervals
#   - Doing 5 month intervals
```


## Miscellaneous Commands

Functionality that some might find useful in non-ordinary usage conditions.

### The Maintenance Log Env Variable

There is an argument for `--maintenance-log-env` (`-e`) which can be used specify the path to the maintenance log via an environment variable. The path specified by this variable will be attempted to be read if the path at the argument `--maintenance-log` (`-m`) cannot be found. If `skip` is passed for the `--maintenance-log-env` argument then the environment will not be attempted to be read for the path to the maintenance log. The `--maintenance-log-env` argument defaults to `MAINTENANCE_LOG`

Some examples:

```sh
# Use the (default) value at the MAINTENANCE_LOG environment variable to try to infer the log path
maintenance_tracker -m non-existent-log-path.toml list

# Use the LOG environment variable to try to infer the log path
maintenance_tracker -m non-existent-log-path.toml -e LOG list

# Skip inferring the maintenance log path with the environment
maintenance_tracker -m non-existent-log-path.toml -e skip list
```

### Verbosity

Verbosity can simply be changed with the `--verbosity` or `-v` flags. Note - the verbosity setting max out at debug. Any number of verbose flags greater than or equal to two will result in debug logs. Examples below.

```sh
# Verbosity at info
./maintenance_tracker --verbose list

# Verbosity at info
./maintenance_tracker -v list

# Verbosity at debug
./maintenance_tracker -vv list

# Verbosity still at debug
./maintenance_tracker -vvvvv list
```

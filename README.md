# cron-trigger

A small Rust binary that fires a single `kvist.v1.Service/Command` gRPC call to a backend service, with Sentry cron check-ins around it. Designed to run as a Kubernetes `CronJob`: one container per scheduled run, one gRPC call, exit.

This is the generalized version of [`digestive`](https://github.com/kvist-no/digestive) — same shape, no digest-specific defaults.

## How it works

On each invocation the binary:

1. Reads required env vars and generates a check-in UUID.
2. Reports `in_progress` to the Sentry cron monitor at `CRON_URL`.
3. Opens a gRPC connection to `NOTIFICATION_SERVICE_URL` and calls `Service/Command` with a `CommandRequest` built from the `COMMAND_*` env vars.
4. Reports `ok` or `error` back to Sentry depending on the gRPC result.

## Configuration

### Required

| Variable | Description |
| --- | --- |
| `ENVIRONMENT` | Environment label sent to Sentry (e.g. `staging`, `production`). |
| `CRON_URL` | Sentry cron monitor check-in URL. Each cron should have its own monitor. |
| `NOTIFICATION_SERVICE_URL` | gRPC endpoint of the target service (e.g. `https://notification.init.svc.cluster.local:8080`). |
| `COMMAND_COMMAND` | Name of the command handler to invoke on the target service. |

### Optional

| Variable | Default | Description |
| --- | --- | --- |
| `COMMAND_FROM` | `Kubernetes Cron Trigger` | Free-text "from" field shown in logs/audit. |
| `COMMAND_DATA` | `{}` | JSON payload passed to the command handler. |
| `COMMAND_REQUESTER` | `""` | Optional requester identifier. |
| `RUST_LOG` | _(unset)_ | Standard `env_logger` filter, e.g. `info`. |

## Deployment

Deployed as Kubernetes CronJobs from [`kvist-no/infra`](https://github.com/kvist-no/infra). Each cron job is a separate `CronJob` manifest pointing at this image with its own `COMMAND_COMMAND`, `COMMAND_DATA`, `CRON_URL`, and (usually) its own ExternalSecret pulling config from GCP Secret Manager.

Example skeleton:

```yaml
kind: CronJob
apiVersion: batch/v1
metadata:
  name: my-cron
  namespace: init
spec:
  schedule: "0 7 * * 1-5"
  timeZone: "Europe/Madrid"
  jobTemplate:
    spec:
      backoffLimit: 0
      template:
        metadata:
          annotations:
            linkerd.io/inject: "enabled"
            config.alpha.linkerd.io/proxy-enable-native-sidecar: "true"
        spec:
          containers:
            - name: cron-trigger
              image: ghcr.io/kvist-no/cron-trigger:sha-XXXXXXX
              envFrom:
                - secretRef:
                    name: my-cron-config
          restartPolicy: Never
```

## Local development

Requires Rust (`rustup`) and `protoc` (`brew install protobuf` on macOS).

Copy `.env.local` and point `NOTIFICATION_SERVICE_URL` at a local backend, then:

```bash
task run
task test
```

# NetApp minimal repro

A minimal reporoduction of issues arising from the use of the `x-id` query parameter when communicating with the NetApp S3 API. 

When run against a NetApp S3 endpoint (as below), the `standard` operation fails whilst the `modified` version succeeds. Whilst running against an AWS or [LocalStack](https://docs.localstack.cloud/user-guide/aws/s3/) S3 bucket succeeds for either of these modes.

```bash
cargo run /
    https://sci-nas-s3.diamond.ac.uk /
    xchemlab-targeting /
    standard /
    --force-path-style /
    --access-key-id "${ACCESS_KEY_ID}" /
    --secret-access-key "${SECRET_ACCESS_KEY}"
```

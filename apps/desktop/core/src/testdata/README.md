# Local proxy TLS fixtures

These certificates and the deliberately public test server private key belong only to the
loopback CONNECT regression test. The CA is trusted by that test client only. The server
certificate has the reserved name `updates.test`; no external service is contacted.
They are valid for 100 years from October 2026. Production certificate checks stay enabled.

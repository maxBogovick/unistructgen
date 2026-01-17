# Service Configuration

This table defines the configuration structure for our microservice.

| Field Name   | Type        | Description                 | Required |
|--------------|-------------|-----------------------------|----------|
| host         | string      | Server hostname             | yes      |
| port         | integer     | Listening port              | yes      |
| db_url       | string      | Database connection string  | yes      |
| max_conn     | integer     | Max DB connections          | no       |
| allowed_ips  | Vec<String> | Whitelisted IPs             | no       |
| feature_flags| json        | Dynamic feature toggles     | yes      |

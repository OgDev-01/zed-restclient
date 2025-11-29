## ADDED Requirements

### Requirement: HTTP Request Execution via Slash Command
The extension SHALL execute HTTP requests from `.http` files via the `/send-request` slash command using Zed's built-in HTTP client.

#### Scenario: Successful request execution
- **WHEN** a user selects an HTTP request block and invokes `/send-request`
- **THEN** the extension SHALL parse the request text
- **AND** execute the request using `zed_extension_api::http_client`
- **AND** return the formatted response as slash command output

#### Scenario: Request with headers
- **WHEN** the HTTP request includes custom headers
- **THEN** all headers SHALL be included in the outgoing request
- **AND** header names and values SHALL be preserved exactly

#### Scenario: Request with body
- **WHEN** the HTTP request includes a body (POST, PUT, PATCH)
- **THEN** the body content SHALL be sent with the request
- **AND** the Content-Type header SHALL be respected

#### Scenario: Empty or invalid request
- **WHEN** the selected text is empty or not a valid HTTP request
- **THEN** the extension SHALL return a descriptive error message
- **AND** the error SHALL indicate what was wrong with the input

### Requirement: HTTP Status Code Limitation Documentation
The extension SHALL clearly document and indicate that HTTP status codes are not available due to Zed HTTP client API limitations.

#### Scenario: Response status display
- **WHEN** an HTTP request completes successfully
- **THEN** the response SHALL indicate status is unknown or assumed
- **AND** the display SHALL show "Status: 200 OK (assumed)" or similar
- **AND** documentation SHALL explain this is a Zed API limitation

#### Scenario: User awareness of limitation
- **WHEN** a user reads the extension documentation
- **THEN** the "Known Limitations" section SHALL clearly state:
  - Zed's HTTP client does not return status codes
  - All successful responses are assumed to be 200 OK
  - Actual server errors may not be distinguishable from success

### Requirement: Response Formatting
The extension SHALL format HTTP responses based on content type for readable display.

#### Scenario: JSON response formatting
- **WHEN** the response Content-Type is `application/json`
- **THEN** the response body SHALL be pretty-printed with indentation
- **AND** syntax highlighting hints SHALL be provided

#### Scenario: XML response formatting
- **WHEN** the response Content-Type is `application/xml` or `text/xml`
- **THEN** the response body SHALL be formatted for readability

#### Scenario: HTML response formatting
- **WHEN** the response Content-Type is `text/html`
- **THEN** the response body SHALL be displayed as-is or with basic formatting

#### Scenario: Plain text response
- **WHEN** the response Content-Type is `text/plain` or unknown
- **THEN** the response body SHALL be displayed verbatim

### Requirement: Request Timeout Handling
The extension SHALL handle request timeouts gracefully and provide feedback to users.

#### Scenario: Request timeout
- **WHEN** an HTTP request exceeds the configured timeout duration
- **THEN** the extension SHALL cancel the request
- **AND** return an error message indicating timeout occurred
- **AND** suggest increasing timeout if appropriate

#### Scenario: Default timeout configuration
- **WHEN** no custom timeout is configured
- **THEN** a reasonable default timeout SHALL be used (e.g., 30 seconds)

### Requirement: Network Error Handling
The extension SHALL handle network errors and provide meaningful error messages.

#### Scenario: Connection refused
- **WHEN** the target server refuses the connection
- **THEN** the extension SHALL return an error indicating connection failure
- **AND** include the target URL in the error message

#### Scenario: DNS resolution failure
- **WHEN** the target hostname cannot be resolved
- **THEN** the extension SHALL return an error indicating DNS failure
- **AND** suggest checking the URL for typos

#### Scenario: SSL/TLS errors
- **WHEN** an SSL/TLS handshake fails
- **THEN** the extension SHALL return an error with SSL context
- **AND** mention certificate validation if relevant

### Requirement: Request Method Support
The extension SHALL support standard HTTP methods through Zed's HTTP client.

#### Scenario: Supported methods
- **WHEN** a request uses GET, POST, PUT, DELETE, PATCH, HEAD, or OPTIONS
- **THEN** the extension SHALL execute the request with the specified method

#### Scenario: Unsupported methods
- **WHEN** a request uses TRACE or CONNECT methods
- **THEN** the extension SHALL return an error indicating the method is not supported
- **AND** explain that Zed's HTTP client does not support these methods

### Requirement: Response Metadata Display
The extension SHALL display relevant response metadata alongside the body.

#### Scenario: Response headers display
- **WHEN** a response is received
- **THEN** response headers SHALL be displayed in the output
- **AND** headers SHALL be formatted as `Header-Name: value`

#### Scenario: Response timing display
- **WHEN** a response is received
- **THEN** the total request duration SHALL be displayed
- **AND** the time SHALL be formatted in a human-readable way (e.g., "245ms")

#### Scenario: Response size display
- **WHEN** a response is received
- **THEN** the response size SHALL be displayed
- **AND** large responses SHALL show size in KB or MB as appropriate

### Requirement: Variable Substitution Before Execution
The extension SHALL substitute variables in the request before execution.

#### Scenario: Environment variable substitution
- **WHEN** a request contains `{{variableName}}` placeholders
- **AND** an environment is active with matching variables
- **THEN** variables SHALL be substituted before the request is sent

#### Scenario: System variable substitution
- **WHEN** a request contains system variables like `{{$timestamp}}` or `{{$guid}}`
- **THEN** the system variables SHALL be resolved to their values
- **AND** the resolved values SHALL be used in the request

#### Scenario: Unresolved variable handling
- **WHEN** a variable cannot be resolved (not defined in environment)
- **THEN** the extension SHALL either:
  - Return an error indicating the undefined variable, OR
  - Leave the placeholder as-is and warn the user
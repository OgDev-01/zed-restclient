# GraphQL Support

The REST Client extension provides comprehensive support for GraphQL queries, mutations, and subscriptions in `.http` and `.rest` files.

## Overview

GraphQL is automatically detected and formatted according to the GraphQL over HTTP specification. The extension handles:

- **Query parsing**: Separates GraphQL operations from variables
- **Syntax validation**: Validates balanced braces, parentheses, and brackets
- **Auto-formatting**: Pretty-prints queries and responses
- **Variable handling**: Supports JSON variables with type checking
- **Error display**: Formats GraphQL errors with location information
- **Syntax highlighting**: GraphQL keywords and structure highlighting

## Basic Usage

### Simple Query (No Variables)

```http
POST https://api.github.com/graphql
Authorization: Bearer YOUR_TOKEN
Content-Type: application/json

query {
  viewer {
    login
    name
    email
  }
}
```

### Query with Variables

Variables are specified as a JSON object after the query, separated by a blank line:

```http
POST https://api.github.com/graphql
Authorization: Bearer YOUR_TOKEN
Content-Type: application/json

query GetUser($login: String!) {
  user(login: $login) {
    id
    name
    bio
    repositories(first: 5) {
      totalCount
      nodes {
        name
        stargazerCount
      }
    }
  }
}

{
  "login": "octocat"
}
```

### Mutations

```http
POST https://api.github.com/graphql
Authorization: Bearer YOUR_TOKEN
Content-Type: application/json

mutation CreateIssue($input: CreateIssueInput!) {
  createIssue(input: $input) {
    issue {
      id
      title
      url
    }
  }
}

{
  "input": {
    "repositoryId": "MDEwOlJlcG9zaXRvcnkxMjk2MjY5",
    "title": "New issue",
    "body": "Issue description"
  }
}
```

## Auto-Detection

GraphQL requests are automatically detected when:

1. **Content-Type header** is `application/graphql` or `application/json`
2. **Body starts with** a GraphQL keyword: `query`, `mutation`, `subscription`, or `fragment`
3. **Shorthand syntax**: Body starts with `{` and contains GraphQL structure

### Example: Auto-Detected (No Content-Type)

```http
POST https://countries.trevorblades.com/graphql

query {
  countries {
    code
    name
  }
}
```

The extension automatically:
- Detects this as GraphQL
- Sets `Content-Type: application/json`
- Formats the request as `{"query": "...", "variables": {...}}`

## Advanced Features

### Named Operations

```http
POST https://api.example.com/graphql
Content-Type: application/json

query GetAllUsers {
  users {
    id
    name
  }
}

mutation CreateUser {
  createUser(input: {name: "John"}) {
    id
  }
}
```

### Fragments

```http
POST https://api.github.com/graphql
Authorization: Bearer YOUR_TOKEN
Content-Type: application/json

query GetRepository($owner: String!, $name: String!) {
  repository(owner: $owner, name: $name) {
    ...RepoInfo
  }
}

fragment RepoInfo on Repository {
  id
  name
  description
  stargazerCount
}

{
  "owner": "facebook",
  "name": "react"
}
```

### Directives

```http
POST https://api.github.com/graphql
Authorization: Bearer YOUR_TOKEN
Content-Type: application/json

query GetUser($login: String!, $includeBio: Boolean!) {
  user(login: $login) {
    login
    name
    bio @include(if: $includeBio)
    company @skip(if: $includeBio)
  }
}

{
  "login": "octocat",
  "includeBio": true
}
```

### Aliases

```http
POST https://api.example.com/graphql
Content-Type: application/json

query {
  user1: user(login: "alice") {
    name
  }
  user2: user(login: "bob") {
    name
  }
}
```

### Nested Variables

```http
POST https://api.example.com/graphql
Content-Type: application/json

mutation CreatePost($input: CreatePostInput!) {
  createPost(input: $input) {
    id
    title
  }
}

{
  "input": {
    "title": "Hello World",
    "content": "Post content",
    "tags": ["intro", "test"],
    "metadata": {
      "category": "blog",
      "published": true
    }
  }
}
```

## Response Formatting

GraphQL responses are automatically detected and formatted with:

### Success Response

```json
{
  "data": {
    "user": {
      "id": "123",
      "name": "John Doe",
      "email": "john@example.com"
    }
  }
}
```

Displays as:

```
# Response Data

{
  "user": {
    "id": "123",
    "name": "John Doe",
    "email": "john@example.com"
  }
}
```

### Error Response

```json
{
  "errors": [
    {
      "message": "Field 'user' not found",
      "locations": [{"line": 2, "column": 5}],
      "path": ["user"]
    }
  ]
}
```

Displays as:

```
# GraphQL Errors

1. Field 'user' not found
   at line 2, column 5
   path: "user"
```

### Response with Extensions

```json
{
  "data": {"user": {"id": "123"}},
  "extensions": {
    "tracing": {
      "duration": 42,
      "startTime": "2023-01-01T00:00:00Z"
    }
  }
}
```

Displays data, errors (if any), and extensions separately.

## Syntax Validation

The extension validates:

- ✅ **Balanced delimiters**: Braces `{}`, parentheses `()`, brackets `[]`
- ✅ **GraphQL keywords**: `query`, `mutation`, `subscription`, `fragment`
- ✅ **Variables format**: Must be valid JSON objects (not arrays or primitives)
- ✅ **String escaping**: Handles strings with special characters correctly

### Valid Examples

```graphql
query { user { id } }                          # Simple query
query GetUser($id: ID!) { user(id: $id) { } } # With variables
mutation { createUser { id } }                 # Mutation
{ users { id } }                               # Shorthand syntax
```

### Invalid Examples (Will Show Errors)

```graphql
query { user { id }      # Missing closing brace
query GetUser($id: ID!   # Missing closing parenthesis
["array"]                # Variables must be objects, not arrays
```

## Public GraphQL APIs for Testing

### Countries API (No Auth Required)

```http
POST https://countries.trevorblades.com/graphql
Content-Type: application/json

query GetCountry($code: ID!) {
  country(code: $code) {
    name
    capital
    currency
    languages {
      name
    }
  }
}

{
  "code": "US"
}
```

### SpaceX API (No Auth Required)

```http
POST https://spacex-production.up.railway.app/graphql
Content-Type: application/json

query GetLaunches($limit: Int!) {
  launches(limit: $limit) {
    mission_name
    launch_date_utc
    rocket {
      rocket_name
    }
  }
}

{
  "limit": 5
}
```

### GitHub GraphQL API (Requires Token)

```http
POST https://api.github.com/graphql
Authorization: Bearer YOUR_GITHUB_TOKEN
Content-Type: application/json

query {
  viewer {
    login
    repositories(first: 5) {
      nodes {
        name
      }
    }
  }
}
```

## Syntax Highlighting

GraphQL syntax highlighting works automatically when:

1. Content-Type is `application/graphql` or `application/json`
2. Body starts with GraphQL keywords
3. Tree-sitter grammar detects GraphQL structure

Highlighted elements include:
- **Keywords**: `query`, `mutation`, `subscription`, `fragment`, `on`
- **Field names**: Query field selections
- **Arguments**: Function arguments and types
- **Variables**: `$variableName`
- **Comments**: `# Comment text`

## Best Practices

### 1. Always Use Variables for Dynamic Values

❌ **Don't** embed values directly:
```graphql
query {
  user(id: "123") {
    name
  }
}
```

✅ **Do** use variables:
```graphql
query GetUser($id: ID!) {
  user(id: $id) {
    name
  }
}

{
  "id": "123"
}
```

### 2. Name Your Operations

❌ **Don't** use anonymous operations:
```graphql
query {
  users { id }
}
```

✅ **Do** name operations:
```graphql
query GetAllUsers {
  users { id }
}
```

### 3. Use Fragments for Reusable Fields

```graphql
query GetUsers {
  activeUsers {
    ...UserFields
  }
  inactiveUsers {
    ...UserFields
  }
}

fragment UserFields on User {
  id
  name
  email
}
```

### 4. Separate Variables Section Clearly

Always use a blank line between query and variables:

```graphql
query GetUser($id: ID!) {
  user(id: $id) { name }
}

{
  "id": "123"
}
```

## Troubleshooting

### "Invalid GraphQL syntax" Error

**Cause**: Unmatched braces, parentheses, or brackets

**Solution**: Check that all `{`, `(`, `[` have matching closing delimiters

### "Variables must be a JSON object" Error

**Cause**: Variables are an array or primitive value

**Solution**: Wrap variables in an object:
```json
{
  "variableName": "value"
}
```

### GraphQL Not Auto-Detected

**Cause**: Missing Content-Type or no GraphQL keywords

**Solution**: Either:
- Add `Content-Type: application/json` header
- Start body with `query`, `mutation`, `subscription`, or `fragment`

### Response Not Formatted as GraphQL

**Cause**: Response doesn't have `data` or `errors` fields

**Solution**: This is expected for non-GraphQL endpoints. The response will be formatted as regular JSON.

## Examples

See [graphql-examples.http](../examples/graphql-examples.http) for comprehensive examples including:

- Simple queries
- Mutations with complex inputs
- Nested variables
- Fragments
- Directives
- Multiple operations
- Public API examples

## References

- [GraphQL Specification](https://spec.graphql.org/)
- [GraphQL over HTTP Specification](https://graphql.github.io/graphql-over-http/)
- [GitHub GraphQL API](https://docs.github.com/en/graphql)
- [Countries GraphQL API](https://countries.trevorblades.com/)
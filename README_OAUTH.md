# Claude OAuth Authentication Flow

This implementation provides a complete OAuth 2.0 authentication flow for Claude, using industry-standard PKCE (Proof Key for Code Exchange) for enhanced security.

## Features

✅ **OAuth 2.0 with PKCE** - Secure authentication using authorization code flow with PKCE
✅ **Secure Token Storage** - Tokens stored in system keychain (macOS Keychain, Windows Credential Manager, Linux Secret Service)
✅ **Automatic Token Refresh** - Seamlessly refreshes expired tokens
✅ **Browser-based Authentication** - Opens browser automatically for user authentication
✅ **Local Callback Server** - Receives OAuth callback on localhost:8989

## Usage

### Login with OAuth

```bash
# Authenticate with Claude using OAuth
oco auth login
```

This will:
1. Open your browser to Claude's OAuth page
2. Start a local server to receive the callback
3. Store tokens securely in your system keychain
4. Configure the CLI to use Claude/Anthropic as the provider

### Check Authentication Status

```bash
# Check if you're authenticated
oco auth status
```

Shows:
- Authentication method (OAuth or API key)
- Token expiration time
- Secure storage availability

### Logout

```bash
# Remove stored OAuth tokens
oco auth logout
```

## Implementation Details

### OAuth Flow

1. **PKCE Generation**: Creates cryptographically secure verifier and challenge
2. **Authorization Request**: Redirects user to Claude OAuth with PKCE challenge
3. **User Authentication**: User logs in through Claude's web interface
4. **Authorization Code**: Claude redirects back with authorization code
5. **Token Exchange**: Exchanges code for access/refresh tokens using PKCE verifier
6. **Secure Storage**: Stores tokens in system keychain

### Security Features

- **PKCE (RFC 7636)**: Prevents authorization code interception attacks
- **State Parameter**: Prevents CSRF attacks
- **Secure Storage**: Uses OS-native secure storage for tokens
- **Token Rotation**: Supports refresh token rotation
- **HTTPS Only**: All OAuth communication over HTTPS

### File Structure

```
src/
├── auth/
│   ├── mod.rs        # Authentication utilities
│   └── oauth.rs      # OAuth client implementation
├── commands/
│   └── auth.rs       # CLI auth commands
└── config/
    └── secure_storage.rs  # Keychain integration
```

## Configuration

The OAuth implementation uses these constants (configurable):

- **Client ID**: `9d1c250a-e61b-44d9-88ed-5944d1962f5e` (Public CLI client)
- **Redirect URI**: `http://localhost:8989/callback`
- **Scopes**: `openid profile email`
- **Token Endpoint**: `https://claude.ai/oauth/token`
- **Authorize Endpoint**: `https://claude.ai/oauth/authorize`

## Token Management

### Access Token
- Used for API requests via `Authorization: Bearer <token>` header
- Expires after a set period (typically 1 hour)
- Automatically refreshed when expired

### Refresh Token
- Used to obtain new access tokens
- Stored securely in system keychain
- Rotated on each refresh for security

### Auto-Refresh Logic
```rust
// Automatically refreshes token if expiring soon
if token_expires_in_less_than_5_minutes {
    refresh_token().await?;
}
```

## Comparison: OAuth vs API Key

| Feature | OAuth | API Key |
|---------|--------|---------|
| Security | ✅ More secure (tokens rotate) | ⚠️ Static credential |
| User Experience | ✅ No manual key management | ❌ Requires manual setup |
| Claude Pro/Max | ✅ Uses subscription | ❌ Separate billing |
| Token Expiry | ✅ Auto-refresh | N/A |
| Revocation | ✅ Can revoke via Claude | ❌ Manual deletion |

## Error Handling

The implementation handles common OAuth errors:

- **Network errors**: Retry with exponential backoff
- **Expired tokens**: Automatic refresh
- **Invalid refresh token**: Prompts re-authentication
- **Timeout**: 5-minute timeout for user authentication
- **Server errors**: Clear error messages with recovery steps

## Testing

Test the OAuth flow:

```bash
# 1. Clear any existing authentication
oco auth logout

# 2. Verify not authenticated
oco auth status

# 3. Login with OAuth
oco auth login

# 4. Verify authentication
oco auth status

# 5. Test with a commit
oco
```

## Troubleshooting

### Browser doesn't open
- Manually visit the URL shown in the terminal
- Check firewall settings for localhost:8989

### Authentication timeout
- Complete authentication within 5 minutes
- Check browser popup blockers

### Token storage fails
- Ensure keychain/credential manager is available
- Run with `--features secure-storage` during build

### Invalid token errors
- Run `oco auth logout` then `oco auth login`
- Check token expiration with `oco auth status`

## Future Enhancements

- [ ] Device code flow for headless environments
- [ ] Multiple account support
- [ ] Token scope management
- [ ] OAuth provider abstraction for other services
- [ ] Configurable token refresh intervals

## Security Considerations

This implementation follows OAuth 2.0 Security Best Practices (RFC 8252):
- Uses PKCE for all flows
- Stores tokens in OS secure storage only
- Never logs or displays tokens
- Implements proper CSRF protection
- Uses cryptographically secure random generation
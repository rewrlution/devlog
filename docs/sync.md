# Sync Feature - MVP Implementation

## Overview

The sync feature enables devlog entries to be synchronized with cloud storage providers. The current MVP implementation provides a foundation for future cloud provider integration while offering immediate functionality through a local file system provider.

## Current Implementation Status ✅

### What's Implemented

1. **Core Architecture**

   - `CloudStorage` trait with async support using `async-trait`
   - Modular provider system for future extensibility
   - Configuration management with TOML files
   - CLI commands for sync operations

2. **Providers**

   - **Local File System**: Simulates cloud storage by copying files to `~/.devlog/sync`
   - **Azure Blob Storage**: Configuration and infrastructure ready (implementation in progress)

3. **Sync Operations**

   - **Push**: Upload local changes to "cloud" (sync directory)
   - **Pull**: Download remote changes to local
   - **Sync**: Bidirectional sync (push + pull)
   - **Status**: Show current configuration and sync state

4. **Configuration**

   - Default config at `~/.devlog/config.toml`
   - Home directory preference (`~/.devlog/entries` for local entries)
   - Support for multiple cloud providers
   - Azure Blob Storage credential management

5. **Safety Features**
   - **No file deletion** - only adds and updates files
   - Last-modified-time conflict resolution
   - Comprehensive error handling with `color-eyre`

## Usage

### Initialize Sync

**For Local Testing:**

```bash
devlog sync init local
```

Creates `~/.devlog/config.toml` with local file system settings.

**For Azure Blob Storage:**

```bash
devlog sync init azure
```

Creates `~/.devlog/config.toml` with Azure Blob Storage template.

### Check Status

```bash
devlog sync status
```

Shows current provider, sync directory, and connection status.

### Sync Operations

```bash
devlog sync push    # Upload local changes
devlog sync pull    # Download remote changes
devlog sync sync    # Bidirectional sync
```

## File Structure

```
src/
├── sync/
│   ├── mod.rs          # CloudStorage trait and core types
│   ├── config.rs       # Configuration management
│   └── engine.rs       # SyncEngine and LocalProvider
└── commands/
    └── sync.rs         # CLI command handlers
```

## Configuration Examples

**Local Provider - `~/.devlog/config.toml`**:

```toml
provider = "local"

[local]
sync_dir = "~/.devlog/sync"
```

**Azure Blob Storage - `~/.devlog/config.toml`**:

```toml
provider = "azure"

[azure]
connection_string = "DefaultEndpointsProtocol=https;AccountName=myaccount;AccountKey=mykey;EndpointSuffix=core.windows.net"
container_name = "devlog-entries"
```

## Azure Blob Storage Setup Guide

### Prerequisites

1. **Azure Account**: You need an active Azure subscription
2. **Storage Account**: Create an Azure Storage Account

### Step 1: Create Azure Storage Account

1. **Go to Azure Portal**: https://portal.azure.com
2. **Create Resource** → **Storage Account**
3. **Fill in details**:
   - **Subscription**: Choose your subscription
   - **Resource Group**: Create new or use existing
   - **Storage Account Name**: Choose a unique name (e.g., `mydevlogstorage`)
   - **Region**: Choose a region close to you
   - **Performance**: Standard
   - **Redundancy**: LRS (Locally Redundant Storage) is sufficient for devlogs

### Step 2: Get Connection String

1. **Go to your Storage Account** in Azure Portal
2. **Security + networking** → **Access Keys**
3. **Copy Connection String** from Key1 or Key2

The connection string looks like:

```
DefaultEndpointsProtocol=https;AccountName=myaccount;AccountKey=abc123...;EndpointSuffix=core.windows.net
```

### Step 3: Configure Devlog

1. **Initialize Azure Config**:

   ```bash
   devlog sync init azure
   ```

2. **Edit Config File**:

   ```bash
   # Edit ~/.devlog/config.toml
   vim ~/.devlog/config.toml
   ```

3. **Replace Connection String**:

   ```toml
   provider = "azure"

   [azure]
   connection_string = "DefaultEndpointsProtocol=https;AccountName=myaccount;AccountKey=abc123...;EndpointSuffix=core.windows.net"
   container_name = "devlog-entries"
   ```

### Step 4: Verify Configuration

```bash
devlog sync status
```

Should show:

```
📊 Sync Status:
  Provider: azure
  Container: devlog-entries
  ✅ Connection string configured
```

### Step 5: Test Sync (When Implementation Complete)

```bash
# Create some test entries
mkdir -p ~/.devlog/entries
echo "# Test Entry" > ~/.devlog/entries/20250920.md

# Sync to Azure
devlog sync push
```

### Security Best Practices

1. **Never commit config files** with real connection strings to version control
2. **Use environment variables** for CI/CD:
   ```bash
   export DEVLOG_AZURE_CONNECTION_STRING="your-connection-string"
   ```
3. **Rotate access keys** regularly in Azure Portal
4. **Use minimal permissions** - only Blob Storage access needed

### Cost Considerations

- **Azure Blob Storage** is very cost-effective for text files
- **LRS redundancy** is sufficient for personal devlogs
- **Hot tier** is recommended for frequently accessed entries
- **Estimated cost**: < $1/month for typical devlog usage (hundreds of markdown files)

### Troubleshooting

**Connection Issues:**

- Verify connection string format
- Check Azure Storage Account exists
- Ensure access keys are not expired

**Container Issues:**

- Container will be created automatically
- Ensure container name follows Azure naming rules (lowercase, no spaces)

**Permission Issues:**

- Verify storage account access keys have full permissions
- Check firewall settings in Azure Storage Account

## Dependencies Added

- `tokio` - Async runtime for file operations
- `toml` - Configuration file parsing
- `async-trait` - Async trait support for dynamic dispatch

## Future Roadmap

### Phase 1: Cloud Providers (Next)

- **Azure Blob Storage**: Complete REST API implementation
- **AWS S3**: Add S3 provider implementation
- **Credential management**: Environment variable support and validation

### Phase 2: Advanced Features

- Conflict resolution strategies
- Performance optimizations
- Error recovery and retries
- Progress reporting for large syncs

### Phase 3: Production Features

- Background sync
- Incremental sync
- Bandwidth optimization
- Multi-device coordination

## Architecture Benefits

### Future-Proof Design

- **Provider Agnostic**: Easy to add Azure, AWS, Google Cloud
- **Modular**: Each component can be enhanced independently
- **Testable**: Mock providers for comprehensive testing
- **Configurable**: Settings can evolve without code changes

### Safety First

- **No Data Loss**: Never deletes files automatically
- **Conflict Resolution**: Deterministic last-modified-wins strategy
- **Rollback Capable**: Can implement transaction-like behavior later
- **Validation**: Input validation prevents corruption

## Technical Details

### Conflict Resolution

- Compare file modification timestamps
- Newer file always wins (overwrites older)
- Preserves modification time when downloading
- No manual merge required

### File Discovery

- Scans `~/.devlog/entries` for `.md` files
- Recursive directory traversal
- Relative path preservation in cloud storage
- UTF-8 filename validation

### Error Handling

- Comprehensive error propagation with `color-eyre`
- Network failure resilience (future)
- Graceful degradation on partial failures
- Clear user feedback and error messages

## Testing Strategy

### Current Testing

- Compilation verification
- Basic CLI command testing
- Configuration file generation
- Status reporting validation

### Planned Testing

- Unit tests for each sync operation
- Integration tests with mock cloud providers
- Performance testing with large file sets
- Multi-device sync simulation
- Network failure recovery testing

## Migration Path

The MVP implementation provides a clear migration path for users:

1. **Start with local sync** to validate workflow
2. **Add cloud provider** by updating config
3. **Existing data preserved** during provider migration
4. **Incremental adoption** of advanced features

This approach minimizes risk while providing immediate value and ensuring the architecture can scale to full cloud sync capabilities.

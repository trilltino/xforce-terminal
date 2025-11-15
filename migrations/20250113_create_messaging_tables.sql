-- Create friendships table for friend requests and relationships
CREATE TABLE IF NOT EXISTS friendships (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    sender_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    receiver_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    status TEXT NOT NULL DEFAULT 'pending' CHECK(status IN ('pending', 'accepted', 'rejected', 'blocked')),
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(sender_id, receiver_id)
);

-- Create index on sender_id for faster lookups
CREATE INDEX IF NOT EXISTS idx_friendships_sender_id ON friendships(sender_id);
-- Create index on receiver_id for faster lookups
CREATE INDEX IF NOT EXISTS idx_friendships_receiver_id ON friendships(receiver_id);
-- Create index on status for filtering
CREATE INDEX IF NOT EXISTS idx_friendships_status ON friendships(status);

-- Create conversation_state table for Braid version tracking
CREATE TABLE IF NOT EXISTS conversation_state (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    conversation_id TEXT NOT NULL UNIQUE,
    user1_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    user2_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    last_version TEXT,
    last_message_at DATETIME,
    user1_unread_count INTEGER NOT NULL DEFAULT 0,
    user2_unread_count INTEGER NOT NULL DEFAULT 0,
    user1_last_read_at DATETIME,
    user2_last_read_at DATETIME,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(user1_id, user2_id)
);

-- Create index on conversation_id for faster lookups
CREATE INDEX IF NOT EXISTS idx_conversation_state_conversation_id ON conversation_state(conversation_id);
-- Create index on user IDs for finding all conversations for a user
CREATE INDEX IF NOT EXISTS idx_conversation_state_user1_id ON conversation_state(user1_id);
CREATE INDEX IF NOT EXISTS idx_conversation_state_user2_id ON conversation_state(user2_id);
-- Create index on last_message_at for sorting
CREATE INDEX IF NOT EXISTS idx_conversation_state_last_message_at ON conversation_state(last_message_at);

-- Create direct_messages table for storing messages
CREATE TABLE IF NOT EXISTS direct_messages (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    sender_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    receiver_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    conversation_id TEXT NOT NULL,
    text TEXT NOT NULL,
    version TEXT UNIQUE,
    timestamp TEXT NOT NULL,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Create index on conversation_id for faster lookups
CREATE INDEX IF NOT EXISTS idx_direct_messages_conversation_id ON direct_messages(conversation_id);
-- Create index on sender_id and receiver_id
CREATE INDEX IF NOT EXISTS idx_direct_messages_sender_id ON direct_messages(sender_id);
CREATE INDEX IF NOT EXISTS idx_direct_messages_receiver_id ON direct_messages(receiver_id);
-- Create index on version for Braid protocol lookups
CREATE INDEX IF NOT EXISTS idx_direct_messages_version ON direct_messages(version);
-- Create index on created_at for chronological ordering
CREATE INDEX IF NOT EXISTS idx_direct_messages_created_at ON direct_messages(created_at);


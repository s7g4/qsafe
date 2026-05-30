-- Create offline_messages table for buffering messages sent to offline users
CREATE TABLE IF NOT EXISTS offline_messages (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    recipient_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    sender_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    content TEXT NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Index for fast lookup by recipient_id when they connect
CREATE INDEX IF NOT EXISTS idx_offline_messages_recipient ON offline_messages(recipient_id);

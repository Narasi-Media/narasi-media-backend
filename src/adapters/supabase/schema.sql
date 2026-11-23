-- ==============================================================================
-- NARASI MEDIA - SUPABASE POSTGRESQL PRODUCTION SCHEMA
-- Deployment Target: Supabase (PostgreSQL 15+)
-- Hexagonal DDD Entity Mapping & RLS Policy
-- ==============================================================================

-- Enable UUID extension
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- 1. USERS TABLE
CREATE TABLE IF NOT EXISTS public.users (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    username TEXT NOT NULL UNIQUE,
    full_name TEXT NOT NULL,
    email TEXT NOT NULL,
    whatsapp_number TEXT NOT NULL,
    telegram_username TEXT,
    password_hash TEXT NOT NULL,
    balance BIGINT NOT NULL DEFAULT 0,
    role TEXT NOT NULL DEFAULT 'Member', -- 'Member', 'Admin', 'Reseller'
    api_key TEXT UNIQUE,
    webhook_url TEXT,
    two_factor_enabled BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Index for fast lookups
CREATE INDEX IF NOT EXISTS idx_users_username ON public.users(username);
CREATE INDEX IF NOT EXISTS idx_users_api_key ON public.users(api_key);

-- 2. BALANCE LOGS (AUDIT TRAIL)
CREATE TABLE IF NOT EXISTS public.balance_logs (
    id TEXT PRIMARY KEY,
    user_id UUID NOT NULL REFERENCES public.users(id) ON DELETE CASCADE,
    log_type TEXT NOT NULL, -- 'debit' or 'credit'
    amount BIGINT NOT NULL,
    initial_balance BIGINT NOT NULL,
    final_balance BIGINT NOT NULL,
    description TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_balance_logs_user_id ON public.balance_logs(user_id);

-- 3. CATEGORIES
CREATE TABLE IF NOT EXISTS public.categories (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    icon TEXT NOT NULL,
    sort_order INT NOT NULL DEFAULT 0
);

-- 4. SERVICES / PRODUCTS
CREATE TABLE IF NOT EXISTS public.services (
    id TEXT PRIMARY KEY,
    category_id TEXT NOT NULL REFERENCES public.categories(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    service_type TEXT NOT NULL DEFAULT 'Default', -- 'Default', 'CustomComments', etc.
    price_original BIGINT NOT NULL DEFAULT 0,
    price_user BIGINT NOT NULL DEFAULT 0,
    min_order INT NOT NULL DEFAULT 10,
    max_order INT NOT NULL DEFAULT 100000,
    refill_supported BOOLEAN NOT NULL DEFAULT false,
    refill_days INT NOT NULL DEFAULT 0,
    description TEXT NOT NULL DEFAULT '',
    provider_id TEXT,
    provider_service_id TEXT,
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_services_category ON public.services(category_id);
CREATE INDEX IF NOT EXISTS idx_services_active ON public.services(is_active);

-- 5. ORDERS
CREATE TABLE IF NOT EXISTS public.orders (
    id TEXT PRIMARY KEY,
    user_id UUID NOT NULL REFERENCES public.users(id) ON DELETE CASCADE,
    service_id TEXT NOT NULL REFERENCES public.services(id),
    target TEXT NOT NULL,
    quantity INT NOT NULL,
    unit_price BIGINT NOT NULL,
    total_price BIGINT NOT NULL,
    status TEXT NOT NULL DEFAULT 'Pending', -- 'Pending', 'Processing', 'Inprogress', 'Success', 'Partial', 'Canceled'
    start_count INT NOT NULL DEFAULT 0,
    remains INT NOT NULL DEFAULT 0,
    provider_order_id TEXT,
    error_message TEXT,
    refill_available BOOLEAN NOT NULL DEFAULT false,
    refill_expires_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_orders_user_id ON public.orders(user_id);
CREATE INDEX IF NOT EXISTS idx_orders_status ON public.orders(status);

-- 6. DEPOSITS
CREATE TABLE IF NOT EXISTS public.deposits (
    id TEXT PRIMARY KEY,
    user_id UUID NOT NULL REFERENCES public.users(id) ON DELETE CASCADE,
    method TEXT NOT NULL, -- 'manual_qris', 'bank_transfer', etc.
    requested_amount BIGINT NOT NULL,
    unique_code INT NOT NULL DEFAULT 0,
    total_to_pay BIGINT NOT NULL,
    balance_credited BIGINT NOT NULL,
    sender_name TEXT,
    proof_image_url TEXT,
    status TEXT NOT NULL DEFAULT 'Pending', -- 'Pending', 'WaitingVerification', 'Success', 'Rejected'
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    approved_at TIMESTAMPTZ,
    rejected_reason TEXT
);

CREATE INDEX IF NOT EXISTS idx_deposits_user_id ON public.deposits(user_id);
CREATE INDEX IF NOT EXISTS idx_deposits_status ON public.deposits(status);

-- ==============================================================================
-- ROW LEVEL SECURITY (RLS) POLICIES
-- Service Role Key has full access; anon/authenticated restricted as needed.
-- ==============================================================================

ALTER TABLE public.users ENABLE ROW LEVEL SECURITY;
ALTER TABLE public.balance_logs ENABLE ROW LEVEL SECURITY;
ALTER TABLE public.categories ENABLE ROW LEVEL SECURITY;
ALTER TABLE public.services ENABLE ROW LEVEL SECURITY;
ALTER TABLE public.orders ENABLE ROW LEVEL SECURITY;
ALTER TABLE public.deposits ENABLE ROW LEVEL SECURITY;

-- Allow backend service role to bypass RLS for all operations
CREATE POLICY "Service Role Full Access Users" ON public.users FOR ALL TO service_role USING (true) WITH CHECK (true);
CREATE POLICY "Service Role Full Access BalanceLogs" ON public.balance_logs FOR ALL TO service_role USING (true) WITH CHECK (true);
CREATE POLICY "Service Role Full Access Categories" ON public.categories FOR ALL TO service_role USING (true) WITH CHECK (true);
CREATE POLICY "Service Role Full Access Services" ON public.services FOR ALL TO service_role USING (true) WITH CHECK (true);
CREATE POLICY "Service Role Full Access Orders" ON public.orders FOR ALL TO service_role USING (true) WITH CHECK (true);
CREATE POLICY "Service Role Full Access Deposits" ON public.deposits FOR ALL TO service_role USING (true) WITH CHECK (true);

-- Public read access for active services & categories
CREATE POLICY "Public Read Categories" ON public.categories FOR SELECT TO anon, authenticated USING (true);
CREATE POLICY "Public Read Active Services" ON public.services FOR SELECT TO anon, authenticated USING (is_active = true);

-- ==============================================================================
-- INITIAL DEMO SEED DATA
-- ==============================================================================

INSERT INTO public.categories (id, name, icon, sort_order)
VALUES 
    ('cat-ig', 'Instagram Followers & Likes', 'instagram', 1),
    ('cat-tiktok', 'TikTok Followers, Likes & Views', 'tiktok', 2),
    ('cat-yt', 'YouTube Subscribers & Watchtime', 'youtube', 3),
    ('cat-tw', 'Twitter / X Retweet & Followers', 'twitter', 4)
ON CONFLICT (id) DO NOTHING;

INSERT INTO public.services (id, category_id, name, service_type, price_original, price_user, min_order, max_order, refill_supported, refill_days, description, is_active)
VALUES
    ('SRV-101', 'cat-ig', 'Instagram Followers REAL Indonesia [Maks 50K] [Garansi 30 Hari]', 'Default', 12000, 15500, 100, 50000, true, 30, 'Followers akun aktif Indonesia berkualitas tinggi. Cocok untuk branding toko & influencer.', true),
    ('SRV-102', 'cat-ig', 'Instagram Likes Indonesia Real High Quality [Super Fast]', 'Default', 4500, 6500, 50, 10000, false, 0, 'Likes instan dalam 1-5 menit dari pengguna aktif Indonesia.', true),
    ('SRV-201', 'cat-tiktok', 'TikTok Followers Indonesia Real [Maks 100K] [Garansi 30 Hari]', 'Default', 18000, 24000, 100, 100000, true, 30, 'Followers TikTok real human. Meningkatkan kredibilitas akun untuk live shopping.', true),
    ('SRV-202', 'cat-tiktok', 'TikTok Views FYP Booster Server Indonesia [Instan]', 'Default', 800, 1500, 1000, 1000000, false, 0, 'Views super cepat untuk memicu algoritma FYP TikTok.', true),
    ('SRV-301', 'cat-yt', 'YouTube Subscribers Organik Indonesia [Garansi Seumur Hidup]', 'Default', 140000, 185000, 50, 5000, true, 90, 'Subscribers aman monetisasi YouTube. Garansi drop refill 90 hari.', true)
ON CONFLICT (id) DO NOTHING;

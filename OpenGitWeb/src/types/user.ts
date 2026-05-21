export interface User {
    id:                   string;
    username:             string;
    display_name:         string | null;
    bio:                  string | null;
    avatar_url:           string | null;
    website:              string | null;
    location:             string | null;
    pronouns:             string | null;
    company:              string | null;
    twitter_username:     string | null;
    role:                 "User" | "Admin" | "Superadmin";
    status_emoji:         string | null;
    status_message:       string | null;
    status_availability:  string;
    is_active:            boolean;
    is_verified:          boolean;
    two_factor_enabled:   boolean;
    profile_private:      boolean;
    created_at:           string;
    updated_at:           string;
}

export interface AuthState {
    user:         User | null;
    access_token: string | null;
    isLoading:    boolean;
}
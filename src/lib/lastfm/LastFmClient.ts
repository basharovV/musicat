import { get } from 'svelte/store';
import { current, userSettings } from '../../data/store';
import md5 from 'md5';

const LASTFM_API_KEY = '7bba19228e6ab143076b84d68205ea8c'; // Public API key for Musicat app
const LASTFM_API_SECRET = '87a4256913b290178c0c2d798985daa4'; // Shared secret
const LASTFM_API_URL = 'https://ws.audioscrobbler.com/2.0/';

interface ScrobbleParams {
  artist: string;
  track: string;
  album?: string;
  albumArtist?: string;
  timestamp?: number;
  duration?: number;
}

interface NowPlayingParams {
  artist: string;
  track: string;
  album?: string;
  albumArtist?: string;
  duration?: number;
}

export class LastFmClient {
  private readonly baseUrl = LASTFM_API_URL;
  private readonly apiKey = LASTFM_API_KEY;
  private readonly apiSecret = LASTFM_API_SECRET;

  private sessionKey: string | null;
  private username: string | null;

  private scrobbleQueue: ScrobbleParams[] = [];
  private isScrobbling = false;

  constructor(sessionKey: string | null = null, username: string | null = null) {
    this.sessionKey = sessionKey;
    this.username = username;
  }

  // Generate the auth signature as per Last.fm API requirements
  private getSignature(params: Record<string, any>): string {
    // Sort keys alphabetically
    const sortedKeys = Object.keys(params).sort();

    // Build signature string
    let signatureStr = '';
    sortedKeys.forEach((key) => {
      if (key !== 'format' && key !== 'callback') {
        signatureStr += key + params[key];
      }
    });

    // Add the shared secret
    signatureStr += this.apiSecret;

    // Calculate MD5 hash
    return md5(signatureStr);
  }

  // Authenticate with username and password via auth.getMobileSession
  async getMobileSession(
    username: string,
    password: string
  ): Promise<{ key: string; name: string }> {
    const method = 'auth.getMobileSession';

    const authParams: Record<string, string> = {
      method,
      api_key: this.apiKey,
      username,
      password,
    };

    const signature = this.getSignature(authParams);

    const body = new URLSearchParams();
    body.append('method', method);
    body.append('api_key', this.apiKey);
    body.append('username', username);
    body.append('password', password);
    body.append('api_sig', signature);
    body.append('format', 'json');

    const response = await fetch(this.baseUrl, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/x-www-form-urlencoded',
      },
      body,
    });

    const text = await response.text();
    if (!response.ok) {
      throw new Error(`Last.fm API error: ${response.status} ${response.statusText}`);
    }

    const data = JSON.parse(text);
    if (data.error) {
      throw new Error(`Last.fm error: ${data.message} (code: ${data.error})`);
    }

    if (!data.session?.key || !data.session?.name) {
      throw new Error('Invalid session structure returned from Last.fm');
    }

    this.sessionKey = data.session.key;
    this.username = data.session.name;

    return {
      key: data.session.key,
      name: data.session.name,
    };
  }

  // Send the "now playing" status to Last.fm
  async updateNowPlaying(params: NowPlayingParams): Promise<boolean> {
    if (!this.sessionKey) return false;

    const method = 'track.updateNowPlaying';
    const requestParams: Record<string, string> = {
      method,
      api_key: this.apiKey,
      sk: this.sessionKey,
      artist: params.artist,
      track: params.track,
      format: 'json',
    };

    if (params.album) requestParams.album = params.album;
    if (params.albumArtist) requestParams.albumArtist = params.albumArtist;
    if (params.duration) requestParams.duration = params.duration.toString();

    requestParams.api_sig = this.getSignature(requestParams);

    const response = await fetch(`${this.baseUrl}?${new URLSearchParams(requestParams)}`, {
      method: 'POST',
    });

    const data = await response.json();
    return response.ok && !data.error;
  }

  // Scrobble a track to Last.fm
  async scrobble(params: ScrobbleParams): Promise<boolean> {
    if (!this.sessionKey) return false;

    const method = 'track.scrobble';
    const requestParams: Record<string, string> = {
      method,
      api_key: this.apiKey,
      sk: this.sessionKey,
      artist: params.artist,
      track: params.track,
      timestamp: (params.timestamp || Math.floor(Date.now() / 1000)).toString(),
      format: 'json',
    };

    if (params.album) requestParams.album = params.album;
    if (params.albumArtist) requestParams.albumArtist = params.albumArtist;
    if (params.duration) requestParams.duration = params.duration.toString();

    requestParams.api_sig = this.getSignature(requestParams);

    const response = await fetch(`${this.baseUrl}?${new URLSearchParams(requestParams)}`, {
      method: 'POST',
    });

    const data = await response.json();
    return response.ok && !data.error;
  }

  // Add a track to the scrobble queue
  queueScrobble(params: ScrobbleParams): void {
    this.scrobbleQueue.push(params);
    this.processQueue();
  }

  // Process the scrobble queue
  private async processQueue(): Promise<void> {
    if (this.isScrobbling || this.scrobbleQueue.length === 0) return;

    this.isScrobbling = true;
    try {
      while (this.scrobbleQueue.length) {
        const params = this.scrobbleQueue.shift();
        if (params) await this.scrobble(params);
      }
    } finally {
      this.isScrobbling = false;
    }
  }
}

// Singleton instance
let lastFmInstance: LastFmClient | null = null;

// Get or create the Last.fm client instance
export function getLastFmClient(): LastFmClient | null {
  const settings = get(userSettings);

  if (!settings.lastfmEnabled || !settings.lastfmSessionKey) {
    return null;
  }

  if (!lastFmInstance) {
    lastFmInstance = new LastFmClient(settings.lastfmSessionKey, settings.lastfmUsername);
  }

  return lastFmInstance;
}

// Initialize a Last.fm client for authentication (doesn't require session key)
export function initLastFmClient(): LastFmClient {
  if (!lastFmInstance) {
    lastFmInstance = new LastFmClient();
  }
  return lastFmInstance;
}

// Authenticate with Last.fm using username and password
export async function authenticateLastFm(
  username: string,
  password: string
): Promise<{ success: boolean; error?: string }> {
  try {
    const client = initLastFmClient();
    const session = await client.getMobileSession(username, password);

    userSettings.update((settings) => {
      settings.lastfmSessionKey = session.key;
      settings.lastfmUsername = session.name;
      settings.lastfmEnabled = true;
      return settings;
    });

    return { success: true };
  } catch (error: any) {
    return { success: false, error: error.message || 'Authentication failed. Please check your credentials.' };
  }
}

// Scrobble the current song
export function scrobbleCurrentSong(): void {
  const client = getLastFmClient();
  if (!client) return;

  const currentSong = get(current).song;
  if (!currentSong) return;

  const params: ScrobbleParams = {
    artist: currentSong.artist || 'Unknown Artist',
    track: currentSong.title || 'Unknown Track',
    timestamp: Math.floor(Date.now() / 1000),
  };

  if (currentSong.album) params.album = currentSong.album;
  if (currentSong.albumArtist) params.albumArtist = currentSong.albumArtist;

  if (currentSong.duration) {
    const parts = currentSong.duration.toString().split(':').map((n) => parseInt(n, 10));
    const seconds = parts.length === 3 ? parts[0] * 3600 + parts[1] * 60 + parts[2] : parts[0] * 60 + parts[1];
    params.duration = seconds;
  }

  client.queueScrobble(params);
}

// Update the "now playing" status for the current song
export function updateNowPlaying(): void {
  const client = getLastFmClient();
  if (!client) return;

  const currentSong = get(current).song;
  if (!currentSong) return;

  const params: NowPlayingParams = {
    artist: currentSong.artist || 'Unknown Artist',
    track: currentSong.title || 'Unknown Track',
  };

  if (currentSong.album) params.album = currentSong.album;
  if (currentSong.albumArtist) params.albumArtist = currentSong.albumArtist;

  if (currentSong.duration) {
    const parts = currentSong.duration.toString().split(':').map((n) => parseInt(n, 10));
    const seconds = parts.length === 3 ? parts[0] * 3600 + parts[1] * 60 + parts[2] : parts[0] * 60 + parts[1];
    params.duration = seconds;
  }

  client.updateNowPlaying(params);
}
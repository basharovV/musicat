// db.ts
import Dexie, { type Table } from 'dexie';
import type { Song } from 'src/App';
import type { SavedSmartQuery } from 'src/lib/smart-query/QueryPart';

export class MySubClassedDexie extends Dexie {
  // 'songs' is added by dexie when declaring the stores()
  // We just tell the typing system this is the case
  songs!: Table<Song>; 
  smartQueries!: Table<SavedSmartQuery>;
  constructor() {
    super('musicatdb');
    this.version(5).stores({
    songs: 'id, title, artist, album, genre, year, duration, [artist+album+trackNumber], [album+trackNumber]', // Primary key and indexed props
    smartQueries: 'name'
    });
  }
}

export const db = new MySubClassedDexie();
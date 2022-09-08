// db.ts
import Dexie, { type Table } from 'dexie';

export class MySubClassedDexie extends Dexie {
  // 'songs' is added by dexie when declaring the stores()
  // We just tell the typing system this is the case
  songs!: Table<Song>; 

  constructor() {
    super('musicatdb');
    this.version(4).stores({
    songs: 'id, title, artist, album, genre, year, duration, [artist+album+trackNumber], [album+trackNumber]' // Primary key and indexed props
    });
  }
}

export const db = new MySubClassedDexie();
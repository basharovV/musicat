<script lang="ts">
import { liveQuery } from "dexie";
import { db } from "../data/db";

//   $: albums = liveQuery(async () => {
//     let results =  db.songs.each($query.orderBy);
//     if ($query.reverse) {
//       results = results.reverse();
//     }
//     let resultsArray = await results.toArray();
//     queriedSongs.set(resultsArray);
//     return resultsArray;
//   });

</script>
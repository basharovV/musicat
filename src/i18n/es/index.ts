import type { BaseTranslation } from "../i18n-types";

const es = {
    infoPopup: {
        builtBy: "Creado por",
        andContributors: "y colaboradores",
        version: "versión",
        releaseNotes: "Notas de la versión",
    },
    sidebar: {
        search: "Buscar",
        library: "Biblioteca",
        albums: "Álbumes",
        favorites: "Favoritos",
        toDelete: "Para borrar",
        playlists: "Listas",
        smartPlaylists: "Listas inteligentes",
        artistsToolkit: "Kit del artista",
        map: "Mapa",
        internetArchive: "Archivo de Internet",
        stats: "Estadísticas",
        openWikiTooltip: "Abrir panel Wiki para {artist}",
        addMetadataHint: "Agregar metadatos",
        takeControl: "Toma el control de tu biblioteca",
        iaMode: "Reproductor principal desactivado en modo Archivo de Internet<br /><br />Detén la reproducción para reactivar",
    },
    library: {
        fields: {
            title: "Título",
            artist: "Artista",
            composer: "Compositor",
            album: "Álbum",
            albumArtist: "Artista del álbum",
            track: "Pista",
            compilation: "Compilación",
            year: "Año",
            dateAdded: "Fecha de añadido",
            genre: "Género",
            origin: "Origen",
            duration: "Duración",
            tags: "Etiquetas",
        },
        resetToFileOrder: "Restablecer al orden de archivos",
        orderHint: "Mostrando orden de archivos",
        orderHintTemp: "Mostrando orden temporal",
        orderDisabledHint: "Cambia al orden de archivos para reordenar pistas",
        queryNoResultsPlaceholder: 'No hay resultados para "{query}"',
    },
    bottomBar: {
        queue: "Cola",
        lyrics: "Letras",
        lossySelector: {
            lossy: "con pérdida",
            lossless: "sin pérdida",
            both: "con y sin pérdida",
        },
        waveform: "",
        nextUp: "Siguiente",
        stats: {
            songs: "canciones",
            artists: "artistas",
            albums: "álbumes",
        },
    },
    smartPlaylists: {
        builtIn: {
            recentlyAdded: "Añadidos recientemente",
            favourites: "Favoritos",
        },
        builder: {
            close: "Cerrar editor",
            save: "Guardar",
            placeholder: "Mi nueva lista inteligente",
            addNewBlock: "añadir nuevo bloque",
            valid: "consulta válida",
            invalid: "consulta inválida",
            parts: {
                byAlbumArtist: {
                    title: "por artista del álbum",
                    example: "ej. por Charlie Parker",
                },
                byArtist: {
                    title: "por artista",
                    example: "ej. por Charlie Parker",
                },
                releasedBetween: {
                    title: "lanzado entre",
                    example: "ej. lanzado entre 1950 y 1967",
                },
                releasedAfter: {
                    title: "lanzado después de",
                    example: "ej. lanzado después de 1950",
                },
                releasedIn: {
                    title: "lanzado en",
                    example: "ej. lanzado en 1999",
                },
                titleContains: {
                    title: "el título contiene {text}",
                    example: "ej. título contiene Amor",
                },
                longerThan: {
                    title: "más largo que",
                    example: "ej. más largo que 04:00",
                },
                containsGenre: {
                    title: "contiene género",
                    example: "ej. contiene Rock",
                },
                fromCountry: {
                    title: "de país",
                    example: "ej. de Australia",
                },
                byComposer: {
                    title: "por compositor",
                    example: "ej. por Charles Mingus",
                },
                containsTag: {
                    title: "contiene etiqueta",
                    example: "ej. contiene amor",
                },
            },
        },
        editSmartPlaylist: "Editar lista inteligente",
        newSmartPlaylist: "Nueva lista inteligente",
        libraryPlaceholder: {
            title: "Los resultados de la lista inteligente aparecerán aquí",
            subtitle: "¡Feliz búsqueda!",
        },
    },
    trackInfo: {
        title: "Información de la pista",
        subtitle: "Usa ARRIBA y ABAJO para cambiar de pista",
        overwriteFile: "Sobrescribir archivo",
        fileInfo: "Información del archivo",
        file: "Archivo",
        codec: "Códec",
        tagType: "Etiqueta",
        duration: "Duración",
        sampleRate: "Frecuencia muestreo",
        bitRate: "Tasa de bits",
        enrichmentCenter: "Centro de enriquecimiento",
        countryOfOrigin: "País de origen",
        countryOfOriginTooltip:
            "Configura esto para usar la vista de mapa y filtrar por país en listas inteligentes",
        fetchingOriginCountry: "Cargando...",
        save: "Guardar",
        fetchFromWikipedia: "Obtener de Wikipedia",
        artworkReadyToSave: "Listo para guardar",
        artworkFound: "Encontrado",
        noArtwork: "Sin imagen",
        artworkTooltip: "Pega la imagen o haz clic para seleccionar un archivo",
        fetchArt: "Buscar imagen",
        metadata: "Metadatos",
        tools: "Herramientas",
        aboutArtwork: "Sobre la imagen",
        artworkTooltipTitle: "🎨 Prioridad de la imagen",
        artworkTooltipBody:
            "<h3 style='margin:0'>🎨 Prioridad de la imagen</h3><br/>Primero, Musicat busca la imagen codificada en los metadatos del archivo, que puedes sobrescribir haciendo clic en este cuadro (se admiten png y jpg). <br/><br/>Si no hay ninguna, buscará un archivo en la carpeta del álbum llamado <i>cover.jpg, folder.jpg</i> o <i>artwork.jpg</i> (puedes cambiar esta lista en Configuración).<br/><br/>De lo contrario, buscará cualquier imagen en la carpeta del álbum y la usará.",
        encodedInFile: "En metadatos",
        bit: "bit",
        noMetadata: "La canción no tiene metadatos",
        unsupportedFormat:
            "Este tipo de archivo aún no es compatible para ver/editar metadatos",
        fix: "Corregir",
        errors: {
            nullChars:
                "Algunas etiquetas tienen un carácter oculto que impide que se lean correctamente.",
        },
        artist: "artista",
        fixLegacyEncodings: {
            title: "Corregir codificaciones antiguas",
            body: "Si tienes etiquetas ID3 codificadas con codificaciones antiguas, deberías actualizarlas a UTF-8 universal para que se muestren correctamente. Selecciona una codificación y haz clic en Corregir.",
            hint: "Selecciona codificación...",
        },
        setTitleFromFileNameHint:
            "Haz clic para establecer título desde el nombre del archivo",
        differentTagTypes:
            "No se pueden editar metadatos de canciones con etiquetas de diferentes tipos",
    },
    settings: {
        title: "Configuración",
        library: "Biblioteca",
        audio: "Audio",
        outputDevice: "Dispositivo de salida",
        followSystem: "Igual que dispositivo del sistema",
        interface: "Interfaz",
        miniPlayerLocation: "",
        language: "",
        features: "Funciones",
        theme: "",
        subtitle: "Configura cosas",
        version: "Versión",
        commaSeparatedFilenames: "nombres de archivo",
        llms: "gpt-3.5-turbo, gpt-4, ollama",
        foldersToWatch: "Carpetas a vigilar",
        folder: "{{1 carpeta | ?? carpetas}}",
        importing: "Importando...",
        coverArtFilenames: "",
        enableArtistsToolkit: "Activar Kit de herramientas del artista",
        import: "",
        enableCoverArtCheck: "",
        songbookLocation: "Ubicación del cancionero",
        scrapbookLocation: "Ubicación del álbum de recortes",
        downloadLocation: "",
        generatedStemsLocation: "",
        playlistsLocation: "",
        enableAIFeatures: "Activar funciones de IA",
        aiModel: "Modelo de IA (LLM)",
        openApiKey: "Clave API de OpenAI",
        geniusApiKey: "Clave API de Genius",
        discogsApiKey: "Clave API de Discogs",
    },
    wiki: {
        inArticle: "Menciones encontradas en tu biblioteca:",
        clickHint: "Haz clic para desplazarte a la mención",
        albums: "Álbumes",
        songs: "Canciones",
        artists: "Artistas",
    },
    tagCloud: {
        close: "Cerrar etiquetas",
    },
    artistsToolkit: {
        header: {
            songbookLocationHint:
                "La ubicación de tu cancionero gestionada por Musicat. Actualmente configurada en {path}",
            songbookLocationHintEmpty:
                "Por favor, configura la ubicación de tu cancionero para que lo gestione Musicat.",
            showScrapbook: "Mostrar álbum de recortes",
            hideScrapbook: "Ocultar álbum de recortes",
        },
        scrapbook: {
            title: "Álbum de recortes",
            setupHint:
                "Por favor, configura la ubicación del álbum de recortes en configuración",
            notFoundError: "Ubicación del álbum de recortes no encontrada",
            openSettings: "Abrir configuración",
            openInFinder: "Abrir en Finder",
        },
        songDetails: {
            tabs: {
                lyrics: "Letras y acordes",
                files: "Archivos",
                other: "Otros",
            },
            lyricsEditor: {
                options: {
                    alignFirstChord: "Alinear primer acorde",
                },
            },
        },
    },
    button: {
        areYouSure: "Haz clic de nuevo para confirmar",
    },
    input: {
        enterHintTooltip: "Presiona ENTER para autocompletar",
    },
    albums: {
        title: "Álbumes",
        options: {
            orderBy: "ordenar por",
            orderByFields: {
                title: "Título",
                artist: "Artista",
                year: "Año",
            },
            showSingles: "sencillos",
            showInfo: "info",
            gridSize: "tamaño de cuadrícula",
        },
        scrollToNowPlaying: "Ir a reproducción actual",
        item: {
            tracksLabel: "pistas",
        },
    },
    trackMenu: {
        reImportTrack: "",
        reImportTracks: "",
        reImportTrackHint: "",
        separateTitle: "",
        separateStems: "",
        editTag: "",
        lookingOnline: "",
        originCountry: "",
        originCountryC: "",
        originCountryHint: "",
        wikiPanel: "",
        removeFromLibrary: "Eliminar {{track | ?? tracks}} de la biblioteca",
        deleteFile: "Eliminar {{file | ?? files}}",
        deleteFileHint: "Mover a la papelera del sistema",
        openInFileManager: "Abrir en {explorerName}",
        infoMetadata: ""
    },
    toDelete: {
        title: "Para borrar",
        description: "Pistas para borrar desde el Modo Poda",
        keepAllBtn: "Conservar todo",
        deleteAllBtn: "Mover todo a la papelera",
        notification: {
            deleting: "Borrando {{track | ?? tracks}}",
            deleted: "Borrado {{track | ?? tracks}}",
        },
    },
    lyrics: {
        autoScroll: "Desplazamiento automático",
    },
    stemSeparation: {
        loading: "",
        complete: "",
        error: "",
        showStems: "",
        cancel: "",
        close: "",
    },
} satisfies BaseTranslation;

export default es;

import type { Translation } from '../i18n-types';

const es = {
	// this is an example Translation, just rename or delete this folder if you want
	tagline: 'Sonido profesional para eventos',
	header: {
		home: 'Inicio',
		aboutUs: 'Acerca de Nosotros',
		contact: 'Contacto'
	},
	services: {
		title: 'Nuestros servicios',
		rental: {
			title: 'Alquiler de Equipos',
			body: 'Proveemos equipos de sonido de primera calidad para asegurar que el audio de su evento sea nítido y claro. Desde micrófonos hasta altavoces, tenemos todo lo que necesita.'
		},
		support: {
			title: 'Soporte para Eventos',
			body: 'Nuestros técnicos experimentados ofrecen soporte en el sitio para asegurar que todo funcione sin problemas, desde la instalación hasta el desmantelamiento.'
		}
	},
	cta: 'Contáctanos'
} satisfies Translation;

export default es;

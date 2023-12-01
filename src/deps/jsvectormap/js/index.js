/**
 * jsVectorMap
 * Copyrights (c) Mustafa Omar https://github.com/themustafaomar
 * Released under the MIT License.
 */
import './util/pollyfills'
import Map from './map'

import '../scss/jsvectormap.scss'

/**
 * ------------------------------------------------------------------------
 * Class Definition
 * ------------------------------------------------------------------------
 */
export default class JsVectorMap {
  constructor(options = {}) {
    if (!options.selector) {
      throw new Error('Selector is not given.')
    }

    return new Map(options)
  }

  // Public
  static addMap(name, map) {
    Map.maps[name] = map
  }
}

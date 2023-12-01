import { merge } from '../util'
import Marker from '../components/marker'

export default function createMarkers(markers = {}, isRecentlyCreated = false) {
  // Create groups for holding markers and markers labels
  // We're checking if `markersGroup` exists or not becuase we may add markers after the map has loaded
  // So we will append the futured markers to this group as well.
  this._markersGroup = this._markersGroup || this.canvas.createGroup('jvm-markers-group')
  this._markerLabelsGroup = this._markerLabelsGroup || this.canvas.createGroup('jvm-markers-labels-group')

  for (let index in markers) {
    const config = markers[index]
    const point = this.getMarkerPosition(config)
    const uid = config.coords.join(':')

    if (!point) {
      continue
    }

    // We're checking if recently created marker does already exist
    // If it does we don't need to create it again, so we'll continue
    // Becuase we may have more than one marker submitted via `addMarkers` method.
    if (isRecentlyCreated) {
      if (
        Object.keys(this._markers).filter(i => this._markers[i]._uid === uid).length
      ) {
        continue
      }

      index = Object.keys(this._markers).length
    }

    const marker = new Marker({
      index,
      map: this,
      // Merge the `markerStyle` object with the marker config `style` if presented.
      style: merge(this.params.markerStyle, { initial: config.style || {} }, true),
      label: this.params.labels && this.params.labels.markers,
      labelsGroup: this._markerLabelsGroup,
      cx: point.x,
      cy: point.y,
      group: this._markersGroup,
      marker: config,
      isRecentlyCreated,
    })

    // Check for marker duplication
    // this is useful when for example: a user clicks a button for creating marker two times
    // so it will remove the old one and the new one will take its place.
    if (this._markers[index]) {
      this.removeMarkers([index])
    }

    this._markers[index] = {
      _uid: uid,
      config: config,
      element: marker,
    }
  }
}
import { createElement } from '../util'
import EventHandler from '../eventHandler'

export default function setupZoomButtons() {
  const zoomin = createElement('div', 'jvm-zoom-btn jvm-zoomin', '&#43;', true)
  const zoomout = createElement('div', 'jvm-zoom-btn jvm-zoomout', '&#x2212', true)

  this.container.appendChild(zoomin)
  this.container.appendChild(zoomout)

  const handler = (zoomin = true) => {
    return () => this._setScale(
      zoomin ? this.scale * this.params.zoomStep : this.scale / this.params.zoomStep,
      this._width / 2,
      this._height / 2,
      false,
      this.params.zoomAnimate
    )
  }

  EventHandler.on(zoomin, 'click', handler())
  EventHandler.on(zoomout, 'click', handler(false))
}
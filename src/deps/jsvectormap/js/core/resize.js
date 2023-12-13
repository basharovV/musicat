export default function resize() {
  const curBaseScale = this._baseScale
  const padding = 50;
  const widthScale = (this._width - padding) / this._defaultWidth;
  const heightScale = (this._height - padding) / this._defaultHeight;

  // if (this._width / this._height > this._defaultWidth / this._defaultHeight) {
  //   this._baseScale = this._height / this._defaultHeight
  //   this._baseTransX = Math.abs(this._width - this._defaultWidth * this._baseScale) / (2 * this._baseScale)
  // } else {
  //   this._baseScale = 
  //   this._baseTransY = Math.abs(this._height - this._defaultHeight * this._baseScale) / (2 * this._baseScale)
  // }
  console.log('wscale', widthScale, 'hscale', heightScale);
  this.scale =  Math.min(widthScale, heightScale);
  this.transX *= this._baseScale / curBaseScale
  this.transY *= this._baseScale / curBaseScale
}
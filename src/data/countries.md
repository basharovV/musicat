Source: https://query.wikidata.org/

```
#List of present-day countries
SELECT DISTINCT ?countryLabel
WHERE
{
  ?country wdt:P31 wd:Q3624078 .
  #not a former country
  FILTER NOT EXISTS {?country wdt:P31 wd:Q3024240}
  #and no an ancient civilisation (needed to exclude ancient Egypt)
  FILTER NOT EXISTS {?country wdt:P31 wd:Q28171280}

  SERVICE wikibase:label { bd:serviceParam wikibase:language "[AUTO_LANGUAGE],mul,en" }
}
ORDER BY ?countryLabel
```

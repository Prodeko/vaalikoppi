# Prodeko äänestyskilke :bookmark:

## Lokaaliajo

Vaatimukset:

- [Docker](https://www.docker.com/)

```bash
$ docker-compose up
```

Tämän jälkeen sivusto pitäisi olla nähtävissä osoitteessa http://localhost:8000/vaalikoppi/.

### Admin

Admin näkymään pääsee osoitteesta http://localhost:8000/vaalikoppi/admin/votings/
![Admin näkymä](admin.png)

Admin-näkymän tunnus on **webbitiimi** ja salasana **kananugetti**.

### Yhteys tietokantaan ja redikseen

Lokaaliajossa tietokantaan ja redikseen saa yhteyden seuraavilla komennoilla:

```
# Tietokanta
$ docker exec -it election_db_1 /bin/sh -c 'psql -h localhost -U vaalikoppi'
vaalikoppi=# \d
...

# Redis
$ docker exec -it election_redis_1 /bin/sh -c 'redis-cli -n 1'
127.0.0.1:6379[1]> KEYS *
...
```

### Testaus

- Käynnistä projekti komennolla `docker-compose up`
- Avaa uusi terminal window
- Testit saa ajettua komennolla `docker exec vaalikoppi pytest election/`
- Testien kattavuus ja rinnakkaisajo `docker exec vaalikoppi pytest --cov -n auto election/`
- Testit ja pdb debugger `docker exec vaalikoppi pytest election/ --pdb`

## Lokaaliajon erityispiirteet Windowsilla

- Juurihakemiston tiedostossa `docker-entrypoint.sh` muuta Windows-tyyliset rivinvaihdot "CRLF" (`\r\n`) Linux-tyylisiksi "LF" (`\n`).
  Rivinvaihtojen ikävyys johtuu siitä, että Windows-koneilla Git tyypillisesti muuntaa rivinvaihdot checkoutissa. Tämän voi muuttaa asetuksista.
- Jos et aja Windowsia admin-käyttäjänä, katso https://icij.gitbook.io/datashare/faq-errors/you-are-not-allowed-to-use-docker-you-must-be-in-the-docker-users-group-.-what-should-i-do
- Jos olet aiemmin käyttänyt tai yrittänyt käyttää Docker Toolboxia, poista Virtualboxista (/vast.) virtuaalikone default ja mahdolliset muut toolbox-liitännäiset koneet.
  Käynnistä Windows uudelleen.
  Jos ei tämän jälkeen vielä toimi: aja Docker Desktopin uninstalleri. Käynnistä Windows uudelleen. Asenna Docker Desktop uudestaan.
- Jos ei vieläkään toimi, `docker-compose stop`, `docker-compose build`, `docker-compose up`

### Deployaus Prodekon palvelimelle

## Azure

1. Kirjaudu Prodekon docker registryyn: `az acr login --name prodekoregistry`
2. Buildaa image: `docker build . -t prodekoregistry.azurecr.io/vaalikoppi/vaalikoppi`
3. Puske image registryyn: `docker push prodekoregistry.azurecr.io/vaalikoppi/vaalikoppi`
4. Aja infrastructure reposta: `ansible-playbook playbook.yml --extra-vars '@passwd.yml' --tags vaalikoppi`

## Äänestysoikeuden myöntäminen ja verifikaatio

Äänestystilaan saapuessa ihmisiltä tarkistaan heidän jäsenyytensä voimassaolo ja heille jaetaan kertakäyttöinen 6-merkkinen koodi, jolla kukin pystyy kirjautumaan äänestysjärjestelmään. Kertakäyttökoodit järjestelmään saa printattua järjestelmän backendistä (jokaisella generointi kerralla syntyy 100 uutta koodia). Koodeja jaettaessa kukin koodi tulee aktivoida äänestyskelpoiseksi järjestelmän backendistä. Muuten koodeilla ei pysty kirjautumaan sisään.

## Äänestäminen

Kirjautumalla järjestelmään jäsenet pääsevät osallistumaan äänestyksiin. Yksi koodi oikeuttaa yhteen ääneen kussakin äänestyksessä ellei äänestyksessä äänestetä tiimin kokoonpanosta, jolloin koodi oikeuttaa äänestyksessä määrättyyn määrään ääniä. Äänestäessä järjestelmä varmistaa vielä, että henkilö on varmasti valinnut haluamansa kandidaatin vahinkoklikkausten välttämiseksi. Jos henkilö ei pysty kirjautumaan saamallaan koodilla järjestelmään, on koodi pahimmassa tapauksessa käytössä jossain muualla. Tällöin on kaikki koodit deaktivoitava sekä jaettava uudet.

## Äänioikeuden poistaminen ja yleiset äänestyspisteet

Henkilön poistuessa tilasta - esimerkiksi vessaan - hänen koodinsa deaktivoidaan, jotta äänestäminen tilan ulkopuolelta estetään. Henkilö pystyy tarkistamaan koodinsa painammalla perusnäkymän yläpalkin sormenjälkeä. Ongelmien ilmetessä on hyvä päivittää sivu ja yrittää uudestaan. Jotta äänestystilaan voidaan järjestää yhteiskäyttöön äänestyspisteitä (läppäreitä tai kännyköitä), jokainen käyttäjä pystyy myös kirjautumaan ulos. Siksi onkin aiheellista, että kukin pitää saamansa koodin tallessa, koska kirjautuminen samalla tunnuksella takaisin ilman koodia on käytännössä mahdotonta (koodit eivät ole linkattu jakamisvaiheessa henkilöihin vaalisalaisuuden vaalimiseksi). Yhteiskäyttöpisteillä kukin käyttäjä kirjautuu järjestelmään sisään, äänestää ja kirjautuu ulos, jolloin piste vapautuu seuraavalle käyttäjälle.

## Äänestysten luominen

Äänestyksiä luodaan backendistä. Backendistä voi määrittää luonnollisesti kunkin äänestyksen ehdokkaat sekä käytettävien äänten määrän (kuinka monta ehdokasta valitaan). Äänestykset voivat olla kolmessa tilassa:

- Avaamattomia: Ehdokkaat ovat näkyvillä, mutta heitä ei voi vielä äänestää
- Avoimia: Ehdokkaita voi äänestää
- Suljettuja: Vain voittanut ehdokas on näkyvillä ja tulokseen ei luonnollisesti voi enää vaikuttaa. Tulokset tulevat näkyville kaikille samaan aikaan.

Äänestyksen suljettua järjestelmä järjestää ehdokkaat äänijakauman mukaan järjestykseen. Tasapelin tai vastaavan tilanteen sattuessa luodaan uusi äänestys, jossa jatkoon valitut ehdokkaat ovat ehdolla.

## Vaalisalaisuus ja yksityisyys

Järjestelmä perustuu pitkälti siihen, että aktiivisten koodien määrä vastaa salissa olevien henkilöiden määrää. Jos näin ei ole, tulee tilanne joko selvittää ja tarvittaessa kaikki koodit deaktivoida ja jakaa uudet. Järjestelmä on suunniteltu tietorakenteltaan niin, että annettuja ääniä ei voi yhdistää jaettuihin koodeihin, jotta kunkin äänestäjän nimettömyydestä voidaan olla varmoja. Järjestelmästä näkee kuitenkin, kuinka monta ääntä on jaettu. Äänestykset suljetaan vasta, kun kaikki salissa olevat ovat jakaneet äänensä.

## Mahdolliset ongelmatilanteet

Teoreettisesti kuka tahansa henkilö voi luovuttaa koodinsa tilan ulkopuolelle. Tämä vastaa kuitenkin tilannetta, jossa joku sanelee toiselle ihmiselle, ketä äänestää. Joka tapauksessa annettujen äänien määrän tulisi vastata tilassa olevien henkilöiden määrää ja siten aktiivisten koodien määrää.

Ongelmatapauksissa tai kysymyksissä ota yhteyttä Prodekon Mediakeisariin mediakeisari@prodeko.org!

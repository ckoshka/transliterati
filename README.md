# transliterati
## what does it do?
You give it this:
```
Барлығына сенімді және тиімді бағдарламалық жасақтаманы құруға мүмкіндік беретін тіл. Ол өте жылдам және жадты үнемдейді: жұмыс уақыты немесе қоқыс жинағышсыз ол өнімділігі маңызды қызметтерді қуаттай алады, ендірілген құрылғыларда жұмыс істей алады және басқа тілдермен оңай біріктіре алады. тың тамаша құжаттамалары, пайдалы қате туралы хабарлары бар ыңғайлы компиляторы және жоғары деңгейлі құралдары — біріктірілген пакет менеджері және құрастыру құралы, автоматты аяқтау және типті тексерулері бар смарт мульти-редакторды қолдау, автоматты пішімдеу және т.б. бар.
```
And this:
```
Barlığına senimdi jäne tïimdi bağdarlamalıq jasaqtamanı qurwğa mümkindik beretin til. Ol öte jıldam jäne jadtı ünemdeydi: jumıs waqıtı nemese qoqıs jïnağışsız ol önimdiligi mañızdı qızmetterdi qwattay aladı, endirilgen qurılğılarda jumıs istey aladı jäne basqa tildermen oñay biriktire aladı. tıñ tamaşa qujattamaları, paydalı qate twralı xabarları bar ıñğaylı kompïlyatorı jäne joğarı deñgeyli quraldarı — biriktirilgen paket menedjeri jäne qurastırw quralı, avtomattı ayaqtaw jäne tïpti tekserwleri bar smart mwltï-redaktordı qoldaw, avtomattı pişimdew
```
And it gives you this:
```
{
  etc...
  "ал": "al",
  "ар": "ar",
  "б": "e",
  "в": "ü",
  "г": "g",
  "д": "d",
  "ді": "di",
  ...etc
}
```
Except it works for any transliteration schema in any language, with some exceptions.
## how fast is it?
The longest newline-separated paragraph constrains its speed, since everything is executed in parallel. Generally it takes between 15ms and 600ms.
## how accurate is it?
It seems to be a matter of:
* How much data do you have? The more the better.
* Is the orthography between the two transliterated pairs is a 1:1 match? Russian is close to perfect even for as little as 14 words, Japanese is only 75% accurate even with 1000 because of the mix of writing systems.
* Are they completely different writing systems? If you pair a logographic language like Chinese with phonetic pinyin, you will need a godawful amount of data.
That's pretty much it.
## how do I use it?
```bash
transliterati file1.txt file2.txt 200
```
Where 200 is the minimum vocab size, if you're really sure you know what you're doing. I think you might have to clone and build it from source since I just learned Rust a week ago and I'm not confident enough with cargo yet.
# Tips:
* If you have a long text, chunk it evenly into pieces if you know where the boundaries are. The longer the chunks are, the longer it will take. The number of chunks doesn't really matter. Make sure there aren't any blank lines.
* Play around with the vocab size if you're getting weird results

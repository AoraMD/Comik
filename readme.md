# Comik

<img src="cover.png" width="256" height="128"/>

<img alt="License" src="https://img.shields.io/github/license/AoraMD/Comik?style=flat-square">

Comik is an automation tool for comic. The tool will create comic PDF document and then send it to the designated email addresses when the monitored comic source is updated, so that you can follow comic on a email-doc reading device like Kindle.

## Example

### Execute

The command "execute" is used to fetch data from source and send created PDF to designated email addresses.

Use `--learn` argument to mark all chapters is read but skip downloading and sending. It is useful if running at the first time.

``` shell
> comik execute --config ./config.json --learn
```

The command uses a [JSON](https://www.json.org/) based configuration file. Take a look in [Guide of Execute](doc/execute.md) for details.

> It is recommended to use timer tools like [crontab](https://man7.org/linux/man-pages/man5/crontab.5.html) to fetch comic update automatically.

### Search

> TODO: Developing

## Notification

Comik may send notifications in situations like comic updates if at least one notify method is configured.

### Bark

[Bark](https://github.com/Finb/Bark) is a custom notification system on iOS. You can use `--bark <url>` to enable and configure Bark notification support. For example:

``` shell
> comik --bark https://api.day.app/exampleKeyXXXXX execute --config ./config.json
```

## Supported source

- [x] dmzj.com
- [ ] ...

## To-Do

- [ ] Parallel working.
- [ ] Comic search.
- [ ] Add notification supported Android.

## Develop purpose

- Following comics on Kindle.
- Learning Rust.

## License

```
MIT License

Copyright (c) 2022 M.D.

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
```


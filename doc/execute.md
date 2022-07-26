# Guide of Execute

## Configuration

### Overview

Configuration of execute command is a JSON file with the format of following sample.

```json
{
    "sender": {
        "address": "sender@example.com",
        "host": "smtp.example.com",
        "password": "passwd"
    },
	"receivers": [
        "kindle@example.com"
    ],
  	"notify": "Comic %comic% has been updated to chapter %chapter% (%success%/%total%).",
  	"source": {}
}
```

- sender: [MailboxObject](#MailboxObject)

    Account that support SMTP protocol used to send documents to designated email addresses.

- receivers: [string]

    Designated email addresses for receiving comic documents.

- notify: string

    > Optional.
    >
    > Default Value: "Comic %comic% has been updated to chapter %chapter% (%success%/%total%)."

    An template string for comic-updating notification content. It will be used if notification is enabled with service like Bark. See [Notification](../readme.md#Notification) in readme.

    There are several token will be replaced while notifying.

    |   token   |                    replacement                    |
    | :-------: | :-----------------------------------------------: |
    |  %comic%  |                updated comic title                |
    | %chapter% |               updated chapter title               |
    | %success% | count of receivers received document successfully |
    |  %total%  |                count of receivers                 |

- source: [SourceObject](#SourceObject)

### MailboxObject

```json
{
    "address": "sender@example.com",
    "host": "smtp.example.com",
    "password": "passwd"
}
```

- address: string

    Sender email address.

- host: string

    SMTP server name.

- password: string

    Sender email account password.

### SourceObject

The object is a K-V pair group, which "K" is source tag and "V" is a source-specific JSON value.

See [Sources](#Sources) for more information.

## Arguments

- `--learn`

    Mark but skip downloading and sending matched chapters. All marked chapters will not be sent on the next run.

- `--scale <factor>`

    > `<factor>` is in range 0.0~1.0.

    The content ratio of comic image in the page. The smaller the factor, the larger the size of white border around the comic image.

## Sources

### DMZJ

The official website of source is www.dmzj.com.

#### Source configuration

```json
{
    "source": {
        "dmzj": [
            {
                "id": "54892"
            },
            {
                "id": "40175"
            }
        ],
    }
}
```

The value type of DMZJ source configuration is a [DmzjChannelObject](#DmzjChannelObject) list. 

#### DmzjChannelObject

```json
{
    "id": "54892"
}
```

- id: string

    Comic ID. It can be obtained by searching or packet capture.

> **Why use object list instead of string list?**
>
> You can add custom elements in the object which will not be processed by JSON parser. It is convenient if trying to add comments for ID.
>
> ```json
> {
>     "id": "54892",
>     "remark": "しあわせ鳥見んぐ"
> }
> ```




export default {
  test: `
<?xml version="1.0" encoding="UTF-8"?>
<rss version="2.0">
  <channel>
    <title>PyPI recent updates for pip</title>
    <link>https://pypi.org/project/pip/</link>
    <description>Recent updates to the Python Package Index for pip</description>
    <language>en</language>
    <item>
      <title>25.0.1</title>
      <link>https://pypi.org/project/pip/25.0.1/</link>
      <description>The PyPA recommended tool for installing Python packages.</description>
      <author>distutils-sig@python.org</author>
      <pubDate>Sun, 09 Feb 2025 17:14:01 GMT</pubDate>
    </item>
    <item>
      <title>25.0</title>
      <link>https://pypi.org/project/pip/25.0/</link>
      <description>The PyPA recommended tool for installing Python packages.</description>
      <author>distutils-sig@python.org</author>
      <pubDate>Sun, 26 Jan 2025 12:40:39 GMT</pubDate>
    </item>
    <item>
      <title>24.3.1</title>
      <link>https://pypi.org/project/pip/24.3.1/</link>
      <description>The PyPA recommended tool for installing Python packages.</description>
      <author>distutils-sig@python.org</author>
      <pubDate>Sun, 27 Oct 2024 18:35:53 GMT</pubDate>
    </item>
    <item>
      <title>24.3</title>
      <link>https://pypi.org/project/pip/24.3/</link>
      <description>The PyPA recommended tool for installing Python packages.</description>
      <author>distutils-sig@python.org</author>
      <pubDate>Sun, 27 Oct 2024 09:45:42 GMT</pubDate>
    </item>
    <item>
      <title>24.2</title>
      <link>https://pypi.org/project/pip/24.2/</link>
      <description>The PyPA recommended tool for installing Python packages.</description>
      <author>distutils-sig@python.org</author>
      <pubDate>Sun, 28 Jul 2024 22:40:52 GMT</pubDate>
    </item>
  </channel>
</rss>
  `,
  expected: [
    '25.0.1',
    '25.0',
    '24.3.1',
    '24.3',
    '24.2'
  ]
}
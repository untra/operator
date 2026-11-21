# frozen_string_literal: true

# Serves every Markdown page's source at `<url>.md` for LLM agents.
module MarkdownTwins
  def self.twin_url(url)
    url == "/" ? "/index.md" : "#{url.chomp("/")}.md"
  end

  def self.header(data)
    [
      data["title"] && "# #{data["title"]}\n\n",
      data["description"] && "> #{data["description"]}\n\n",
    ].compact.join
  end

  class Generator < Jekyll::Generator
    safe true
    priority :lowest

    def generate(site)
      sources = site.pages.select { |p| p.ext == ".md" && p.output_ext == ".html" }
      site.pages.concat(sources.map { |page| twin(site, page) })
    end

    private

    def twin(site, page)
      url = MarkdownTwins.twin_url(page.url)
      page.data["markdown_url"] = url
      Jekyll::PageWithoutAFile.new(site, site.source, File.dirname(page.relative_path), "#{page.basename}.mdtwin").tap do |t|
        t.content = MarkdownTwins.header(page.data) + page.content
        t.data.merge!("permalink" => url, "layout" => nil, "sitemap" => false)
      end
    end
  end
end

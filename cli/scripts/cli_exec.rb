#!/usr/bin/ruby

require 'json'
require 'erb'

template_file = ARGV[0]
json_str = STDIN.read

@json = JSON.parse(json_str)

template = ERB.new(File.read(template_file))
puts template.result


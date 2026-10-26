require 'json'
require 'digest'
require 'open3'
require 'time'
base=ARGV.shift
raise 'output base and command required' unless base && !ARGV.empty?
raise 'receipt exists' if File.exist?(base+'.json')
revision,_,gs=Open3.capture3('git','rev-parse','HEAD')
started=Time.now.utc.iso8601(6)
out,err,status=Open3.capture3(*ARGV)
File.binwrite(base+'.stdout',out)
File.binwrite(base+'.stderr',err)
File.write(base+'.json',JSON.pretty_generate({started:started,finished:Time.now.utc.iso8601(6),cwd:Dir.pwd,revision:gs.success? ? revision.strip : nil,argv:ARGV,runner_sha256:Digest::SHA256.file(__FILE__).hexdigest,exit_code:status.exitstatus,stdout:{path:base+'.stdout',bytes:out.bytesize,sha256:Digest::SHA256.hexdigest(out)},stderr:{path:base+'.stderr',bytes:err.bytesize,sha256:Digest::SHA256.hexdigest(err)}})+"\n")
puts out
warn err unless err.empty?
exit status.exitstatus
